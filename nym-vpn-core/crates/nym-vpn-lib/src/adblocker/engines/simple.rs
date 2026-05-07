// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use adblock::lists::{ParseOptions, ParsedFilter, RuleTypes, parse_filter};
use futures::{StreamExt, TryFutureExt};
use nym_common::trace_err_chain;
use nym_sqlx_pool_guard::SqlitePoolGuard;
use sqlx::{
    Connection, Sqlite, SqliteConnection, SqlitePool,
    pool::PoolConnection,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use time::OffsetDateTime;
use tokio::{fs, sync::RwLock};

use crate::{
    adblocker::{
        AdBlockerError, Result,
        engines::AdBlockEngine,
        file_manager::{SOURCES, Source, SourceMetaData},
    },
    dns_filter::{DnsFilterDecision, DnsFilterStrategy, DnsFilterT},
};

/// Soft heap limit that advises SQLite to free up memory
const SQL_SOFT_HEAP_LIMIT: usize = 4 * 1024 * 1024;
/// Hard heap limit that enforces a strict ceiling on total heap memory usage
const SQL_HARD_HEAP_LIMIT: usize = 5 * 1024 * 1024;

/// Ad-block engine that uses SQLite database for storing the blocklist of domains.
#[derive(Clone)]
pub struct SimpleAdBlockEngine {
    db_path: PathBuf,
    db: Arc<RwLock<Option<SqlitePoolGuard>>>,
}

impl SimpleAdBlockEngine {
    // Allow dead code for tests to run on desktop
    #[allow(dead_code)]
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            db: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl AdBlockEngine for SimpleAdBlockEngine {
    async fn load_filters(&self, dir: &Path) -> Result<()> {
        let db = {
            let mut db_guard = self.db.write().await;
            match db_guard.as_ref() {
                Some(db) => db.clone(),
                None => {
                    let db = open_db(&self.db_path)
                        .or_else(|err| async move {
                            trace_err_chain!(err, "failed to open adblock db");
                            tracing::info!("Rebuild adblock db from seed");

                            if let Err(err) = remove_db(&self.db_path).await {
                                trace_err_chain!(err, "failed to remove database");
                            }

                            open_db(&self.db_path).await
                        })
                        .await?;

                    let _ = db_guard.replace(db.clone());
                    db
                }
            }
        };

        let conn = (*db)
            .acquire()
            .await
            .map_err(AdBlockerError::AcquireDbConnection)?;

        populate_db(dir, conn).await?;

        Ok(())
    }

    async fn unload_filters(&self) {
        let mut db_guard = self.db.write().await;

        if let Some(db) = db_guard.take() {
            db.close().await;
        }
    }
}

#[async_trait::async_trait]
impl DnsFilterT for SimpleAdBlockEngine {
    async fn should_block(&self, domain: &str) -> DnsFilterDecision {
        const PASS: DnsFilterDecision = DnsFilterDecision::Pass;
        const BLOCK: DnsFilterDecision = DnsFilterDecision::Block(DnsFilterStrategy::Localhost);

        let domain = domain
            .trim()
            .trim_end_matches('/')
            .trim_end_matches('.');

        // Treat empty / root as non-blockable.
        if domain.is_empty() {
            return PASS;
        }

        // Always pass if database is not loaded
        let db_guard = self.db.read().await;
        let Some(db) = &*db_guard else {
            return PASS;
        };

        // Lowercase for case-insensitive lookup
        let domain = domain.to_lowercase();

        let should_block = db
            .acquire()
            .and_then(|conn| async move { DbRequest::new(conn).has_domain(&domain).await })
            .await
            .unwrap_or(false);

        if should_block { BLOCK } else { PASS }
    }
}

#[derive(Clone)]
struct DbRequest<T>
where
    T: AsMut<SqliteConnection>,
{
    executor: T,
}

impl<T> DbRequest<T>
where
    T: AsMut<SqliteConnection>,
{
    fn new(executor: T) -> Self {
        Self { executor }
    }

    fn into_inner(self) -> T {
        self.executor
    }
}

impl<T> DbRequest<T>
where
    T: AsMut<SqliteConnection>,
{
    /// Returns true if the update should be skipped (i.e., the data is already up-to-date).
    /// The check is done by just finding the first entry with the given source ID and timestamp.
    /// This should suffice since updates happen in bulk using transactions.
    pub async fn should_skip_update(
        &mut self,
        source_id: &str,
        update_timestamp: OffsetDateTime,
    ) -> sqlx::Result<bool> {
        let update_timestamp = update_timestamp.unix_timestamp();
        sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM blocked_domains WHERE source = $1 AND updated_at = $2 LIMIT 1)"#,
            source_id,
            update_timestamp
        )
        .fetch_one(self.executor.as_mut())
        .map_ok(|res| res == 1)
        .await
    }

    /// Add domain to the database if it doesn't exist yet.
    pub async fn add_domain(
        &mut self,
        domain: String,
        source_id: &str,
        update_timestamp: OffsetDateTime,
    ) -> sqlx::Result<()> {
        let update_timestamp = update_timestamp.unix_timestamp();
        sqlx::query!(
            r#"INSERT INTO blocked_domains (domain_name, source, updated_at) VALUES ($1, $2, $3)
            ON CONFLICT(domain_name, source) DO UPDATE SET updated_at = $3 WHERE updated_at != $3"#,
            domain,
            source_id,
            update_timestamp
        )
        .execute(self.executor.as_mut())
        .await?;
        Ok(())
    }

    /// Remove domains that match the given source, but not the given timestamp.
    /// This is used to clean up stale entries that are no longer a part of the new blocklist.
    pub async fn remove_stale_domains(
        &mut self,
        source_id: &str,
        update_timestamp: OffsetDateTime,
    ) -> sqlx::Result<()> {
        let update_timestamp = update_timestamp.unix_timestamp();
        sqlx::query!(
            "DELETE FROM blocked_domains WHERE source = $1 AND updated_at != $2",
            source_id,
            update_timestamp
        )
        .execute(self.executor.as_mut())
        .await?;

        Ok(())
    }

    /// Returns true if the given domain is blocked.
    pub async fn has_domain(&mut self, domain: &str) -> sqlx::Result<bool> {
        sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM blocked_domains WHERE domain_name = $1 LIMIT 1)"#,
            domain
        )
        .fetch_one(self.executor.as_mut())
        .map_ok(|res| res == 1)
        .await
    }
}

/// Open the SQLite database at the given path and perform migrations.
async fn open_db(db_path: &Path) -> Result<SqlitePoolGuard> {
    let opts = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .statement_cache_capacity(100)
        .pragma("soft_heap_limit", SQL_SOFT_HEAP_LIMIT.to_string())
        .pragma("hard_heap_limit", SQL_HARD_HEAP_LIMIT.to_string());

    let pool = SqlitePool::connect_with(opts)
        .await
        .map_err(AdBlockerError::OpenDb)?;
    let pool = SqlitePoolGuard::new(pool);

    match sqlx::migrate!("src/adblocker/db/migrations")
        .run(&*pool)
        .await
        .map_err(AdBlockerError::MigrateDb)
    {
        Ok(_) => Ok(pool),
        Err(err) => {
            pool.close().await;
            Err(err)
        }
    }
}

/// Populate the database from compressed blocklists on disk.
/// It does nothing if the database is already populated.
async fn populate_db(cache_dir: &Path, mut conn: PoolConnection<Sqlite>) -> Result<()> {
    for source in SOURCES.iter() {
        let data_path = cache_dir.join(source.file_name);

        let opts = ParseOptions {
            format: source.filterset_format,
            rule_types: RuleTypes::NetworkOnly,
            ..Default::default()
        };

        let meta_path = cache_dir.join(source.meta_file_name);
        let meta_data = SourceMetaData::from_file(&meta_path).await?;

        let update_timestamp = meta_data.updated_utc;
        let source_id = source.file_name;

        // Quick check to skip update if data hasn't changed
        let mut db_request = DbRequest::new(&mut conn);
        let skip_update = db_request
            .should_skip_update(source_id, update_timestamp)
            .await
            .map_err(AdBlockerError::PopulateDb)?;

        if skip_update {
            tracing::debug!("Skip update for {source_id} (updated at {update_timestamp})");
            continue;
        }

        let trans = conn.begin().await.map_err(AdBlockerError::PopulateDb)?;
        let mut db_request = DbRequest::new(trans);

        let mut lines = Source::stream_lines(&data_path);
        while let Some(line) = lines.next().await {
            let line = line?;

            // Ignore errors since they aren't that useful
            if let Ok(ParsedFilter::Network(filter)) = parse_filter(&line, false, opts)
                && let Some(ref domain) = filter.hostname
            {
                // Convert to lowercase for case-insensitive comparison
                let domain = domain.to_lowercase();

                db_request
                    .add_domain(domain, source_id, update_timestamp)
                    .await
                    .map_err(AdBlockerError::PopulateDb)?;
            }
        }

        // Remove entries that haven't been updated
        db_request
            .remove_stale_domains(source_id, update_timestamp)
            .await
            .map_err(AdBlockerError::PopulateDb)?;

        let trans = db_request.into_inner();
        trans.commit().await.map_err(AdBlockerError::PopulateDb)?;
    }

    Ok(())
}

/// Remove the database file and associated WAL/SHM files.
async fn remove_db(db_path: &Path) -> std::io::Result<()> {
    let paths = vec![
        db_path.to_path_buf(),
        add_path_suffix(db_path, "-wal"),
        add_path_suffix(db_path, "-shm"),
    ];

    let results = futures::stream::iter(paths)
        .then(|path| async move {
            fs::remove_file(&path).await.or_else(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(err)
                }
            })
        })
        .collect::<Vec<Result<(), std::io::Error>>>()
        .await;

    let first_err = results.into_iter().find(|r| r.is_err());

    first_err.unwrap_or(Ok(()))
}

fn add_path_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut new_path = path.to_path_buf();
    new_path.as_mut_os_string().push(suffix);
    new_path
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::adblocker::file_manager::tests::init_tests;

    const SHOULD_BE_BLOCKED_DOMAIN: &str = "ad.doubleclick.net";

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_blocks_domain() {
        let temp_dir = init_tests().await.unwrap();
        let engine = SimpleAdBlockEngine::new(temp_dir.path().join("adblock.db"));

        engine.load_filters(temp_dir.path()).await.unwrap();
        assert!(matches!(
            engine.should_block(SHOULD_BE_BLOCKED_DOMAIN).await,
            DnsFilterDecision::Block(_)
        ));

        engine.unload_filters().await;
        let decision = engine.should_block(SHOULD_BE_BLOCKED_DOMAIN).await;
        assert!(matches!(decision, DnsFilterDecision::Pass));
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_should_not_block_without_rules() {
        let temp_dir = TempDir::new().unwrap();
        let engine = SimpleAdBlockEngine::new(temp_dir.path().join("adblock.db"));
        let decision = engine.should_block(SHOULD_BE_BLOCKED_DOMAIN).await;
        assert!(matches!(decision, DnsFilterDecision::Pass));
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_remove_db_files() {
        let temp_dir = TempDir::new().unwrap();

        let files = ["adblock.db", "adblock.db-wal", "adblock.db-shm"]
            .into_iter()
            .map(|f| temp_dir.path().join(f))
            .collect::<Vec<_>>();

        tracing::debug!("creating db files: {:?}", files);

        for f in files.iter() {
            fs::File::create(f).await.unwrap();
        }

        remove_db(&temp_dir.path().join("adblock.db"))
            .await
            .unwrap();

        for f in files {
            assert!(
                matches!(fs::metadata(f).await, Err(err) if err.kind() == std::io::ErrorKind::NotFound)
            )
        }
    }
}
