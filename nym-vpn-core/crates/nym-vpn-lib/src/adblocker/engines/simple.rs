// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use adblock::{
    filters::network::NetworkFilterMaskHelper,
    lists::{ParseOptions, ParsedFilter, RuleTypes, parse_filter},
};
use futures::{StreamExt, TryFutureExt, TryStreamExt, pin_mut};
use itertools::Itertools;
use nym_common::trace_err_chain;
use nym_sqlx_pool_guard::SqlitePoolGuard;
use sqlx::{
    ConnectOptions, Connection, QueryBuilder, Sqlite, SqliteConnection,
    pool::PoolConnection,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use time::OffsetDateTime;
use tokio::{fs, sync::RwLock};

use crate::{
    adblocker::{
        AdBlockerError, Result,
        engines::AdBlockEngine,
        file_manager::{SOURCES, Source},
    },
    dns_filter::{DnsFilterDecision, DnsFilterStrategy, DnsFilterT},
};

/// Soft heap limit that advises SQLite to free up memory
const SQL_SOFT_HEAP_LIMIT: usize = 7 * 1024 * 1024;
/// Hard heap limit that enforces a strict ceiling on total heap memory usage
const SQL_HARD_HEAP_LIMIT: usize = 8 * 1024 * 1024;
/// Minimum number of idle connections to the database
const SQL_MIN_CONNECTIONS: u32 = 10;
/// Maximum number of concurrent connections to the database
const SQL_MAX_CONNECTIONS: u32 = 20;
/// Number of domains to insert in a single batch
const DOMAIN_BATCH_SIZE: usize = 999;

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
                            tracing::info!("Recreate adblock db");

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

        let domain = super::qname_to_domain_name(domain);

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

struct DomainsBatch<'a> {
    domains: &'a [String],
    source_id: &'a str,
    updated_at: OffsetDateTime,
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

    /// Add domains to the database if they don't exist yet.
    /// This method performs a batch update for performance.
    pub async fn add_domains_batch<'a>(&mut self, batch: DomainsBatch<'a>) -> sqlx::Result<()> {
        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("INSERT INTO blocked_domains (domain_name, source, updated_at) ");

        let update_timestamp = batch.updated_at.unix_timestamp();
        query_builder.push_values(batch.domains, |mut b, domain| {
            b.push_bind(domain)
                .push_bind(batch.source_id)
                .push_bind(update_timestamp);
        });

        query_builder.push(
            r#" ON CONFLICT(domain_name, source) DO UPDATE SET
                updated_at = excluded.updated_at
                WHERE updated_at != excluded.updated_at"#,
        );

        let query = query_builder.build();
        query.execute(self.executor.as_mut()).await?;

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
        let domains: Vec<String> = get_all_subdomains(domain);
        if domains.is_empty() {
            return Ok(false);
        }

        let placeholders = vec!["?"; domains.len()].join(", ");
        let sql = format!(
            "SELECT EXISTS(SELECT 1 FROM blocked_domains WHERE domain_name IN ({}) LIMIT 1)",
            placeholders
        );

        let mut query = sqlx::query_scalar(&sql);
        for d in domains {
            query = query.bind(d);
        }

        query
            .fetch_one(self.executor.as_mut())
            .map_ok(|res: i32| res == 1)
            .await
    }
}

/// Open the SQLite database at the given path and perform migrations.
async fn open_db(db_path: &Path) -> Result<SqlitePoolGuard> {
    let opts = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .disable_statement_logging()
        .pragma("soft_heap_limit", SQL_SOFT_HEAP_LIMIT.to_string())
        .pragma("hard_heap_limit", SQL_HARD_HEAP_LIMIT.to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(SQL_MAX_CONNECTIONS)
        .min_connections(SQL_MIN_CONNECTIONS)
        .connect_with(opts)
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

        let mtime = tokio::fs::metadata(&data_path)
            .await
            .and_then(|m| m.modified())
            .map(OffsetDateTime::from)
            .unwrap_or_else(|_| OffsetDateTime::now_utc());

        let update_timestamp = mtime;
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

        let start = tokio::time::Instant::now();
        let trans = conn.begin().await.map_err(AdBlockerError::PopulateDb)?;
        let mut db_request = DbRequest::new(trans);

        let line_stream = Source::stream_lines(&data_path);
        let chunk_stream = line_stream
            .try_filter_map(|line| async move {
                // Ignore errors since they aren't that useful
                let Ok(ParsedFilter::Network(filter)) = parse_filter(&line, false, opts) else {
                    return Ok(None);
                };

                let Some(ref domain) = filter.hostname else {
                    return Ok(None);
                };

                // Only support rules blocking by domain (double pipe)
                // See: https://adblockplus.org/filter-cheatsheet
                if filter.is_hostname_anchor() {
                    // Convert to lowercase for case-insensitive comparison
                    Ok(Some(domain.to_lowercase()))
                } else {
                    Ok(None)
                }
            })
            .try_chunks(DOMAIN_BATCH_SIZE)
            .map_err(|try_chunks_err| try_chunks_err.1);
        pin_mut!(chunk_stream);

        while let Some(result) = chunk_stream.next().await {
            let domains = result?;

            db_request
                .add_domains_batch(DomainsBatch {
                    domains: &domains,
                    source_id,
                    updated_at: update_timestamp,
                })
                .await
                .map_err(AdBlockerError::PopulateDb)?;
        }

        // Remove entries that haven't been updated
        db_request
            .remove_stale_domains(source_id, update_timestamp)
            .await
            .map_err(AdBlockerError::PopulateDb)?;

        let trans = db_request.into_inner();
        trans.commit().await.map_err(AdBlockerError::PopulateDb)?;

        let duration = start.elapsed();
        tracing::debug!("Populated database from {source_id} in {duration:?}");
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

fn get_all_subdomains(domain: &str) -> Vec<String> {
    let components = domain
        .split('.')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>();

    if components.len() > 1 {
        let mut matches = Vec::with_capacity(8);
        // skip top-level domain
        for skip in 0..components.len() - 1 {
            let subdomain = components.iter().skip(skip).join(".");
            matches.push(subdomain);
        }
        matches
    } else {
        Vec::from_iter(components.first().map(|v| (*v).to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::adblocker::file_manager::tests::init_tests;

    const SHOULD_BE_BLOCKED_DOMAIN: &str = "ad.doubleclick.net";

    #[test]
    fn test_get_all_subdomains() {
        assert_eq!(
            get_all_subdomains("ad.doubleclick.net"),
            vec!["ad.doubleclick.net", "doubleclick.net"]
        );
        assert_eq!(
            get_all_subdomains(".ad..doubleclick.net."),
            vec!["ad.doubleclick.net", "doubleclick.net"]
        );
        assert_eq!(get_all_subdomains("localhost"), vec!["localhost"]);
        assert_eq!(get_all_subdomains("nym.com"), vec!["nym.com"]);
        assert_eq!(get_all_subdomains(""), Vec::<&str>::new());
    }

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

    #[tokio::test]
    async fn test_add_batch_domains() {
        let temp_dir = TempDir::new().unwrap();

        let domains: Vec<String> = (0..100_000).map(|i| format!("domain-{}.com", i)).collect();
        let db = open_db(&temp_dir.path().join("adblock.db")).await.unwrap();

        let mut req = DbRequest::new(db.acquire().await.unwrap());
        for chunk in domains.chunks(DOMAIN_BATCH_SIZE) {
            let batch = DomainsBatch {
                domains: chunk,
                source_id: "bench",
                updated_at: OffsetDateTime::from_unix_timestamp(1768163200).unwrap(),
            };
            req.add_domains_batch(batch).await.unwrap();
        }

        let start = tokio::time::Instant::now();
        let trans = db.begin().await.unwrap();
        let mut req = DbRequest::new(trans);
        let update_timestamp = OffsetDateTime::from_unix_timestamp(1778163200).unwrap();
        for chunk in domains.chunks(DOMAIN_BATCH_SIZE) {
            let batch = DomainsBatch {
                domains: chunk,
                source_id: "bench",
                updated_at: update_timestamp,
            };
            req.add_domains_batch(batch).await.unwrap();
        }
        req.into_inner().commit().await.unwrap();

        let duration = start.elapsed();
        println!("Total time: {:?}", duration);
        println!("Rows per second: {:.2}", 100_000.0 / duration.as_secs_f64());

        let mut conn = db.acquire().await.unwrap();
        let update_timestamp = update_timestamp.unix_timestamp();
        let entry_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM blocked_domains WHERE updated_at = $1"#,
            update_timestamp
        )
        .fetch_one(conn.as_mut())
        .await
        .unwrap();
        assert_eq!(entry_count, 100_000);
    }
}
