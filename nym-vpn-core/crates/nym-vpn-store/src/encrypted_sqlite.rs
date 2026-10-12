// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use sqlx::{
    AssertSqlSafe, Connection, Executor, SqliteConnection,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};

const SQLITE_JOURNAL_SUFFIXES: [&str; 3] = ["-wal", "-shm", "-journal"];

#[derive(thiserror::Error, Debug)]
pub enum EncryptSqliteStoreError {
    #[error("failed to check whether database file exists")]
    CheckDatabaseExists(#[source] std::io::Error),

    #[error("failed to remove database file: {database_path}")]
    RemoveDatabaseFile {
        database_path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("source database is encrypted with different key or not a database")]
    NotADatabase,

    #[error("failed to move database file: {from} -> {to}")]
    MoveDatabaseFile {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("SQL error: {ctx}")]
    Sql {
        ctx: &'static str,
        #[source]
        source: sqlx::Error,
    },
}

pub enum MigrationResult {
    /// Database file does not exist
    NoDatabaseExist,
    /// Database is already encrypted
    AlreadyEncrypted,
    /// Database was encrypted
    Completed,
}

impl std::fmt::Display for MigrationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NoDatabaseExist => "database file does not exist",
            Self::AlreadyEncrypted => "database is already encrypted",
            Self::Completed => "migration completed, database is encrypted now",
        })
    }
}

pub async fn encrypt_sqlite_store(
    path: &Path,
    journal_mode: SqliteJournalMode,
    key: &str,
) -> Result<MigrationResult, EncryptSqliteStoreError> {
    if !tokio::fs::try_exists(path)
        .await
        .map_err(EncryptSqliteStoreError::CheckDatabaseExists)?
    {
        Ok(MigrationResult::NoDatabaseExist)
    } else if try_open_encrypted(path, journal_mode, key).await? {
        Ok(MigrationResult::AlreadyEncrypted)
    } else {
        try_migrate_store(path, journal_mode, key)
            .await
            .map(|_| MigrationResult::Completed)
    }
}

async fn try_open_encrypted(
    path: &Path,
    journal_mode: SqliteJournalMode,
    key: &str,
) -> Result<bool, EncryptSqliteStoreError> {
    let encrypted_options = SqliteConnectOptions::default()
        .filename(path)
        .journal_mode(journal_mode)
        .pragma("key", quote_key(key));

    if let Some(conn) =
        try_open(&encrypted_options)
            .await
            .map_err(|source| EncryptSqliteStoreError::Sql {
                ctx: "open encrypted database",
                source: source,
            })?
    {
        conn.close()
            .await
            .map_err(|source| EncryptSqliteStoreError::Sql {
                ctx: "close encrypted database",
                source,
            })?;
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn try_migrate_store(
    database_path: &Path,
    journal_mode: SqliteJournalMode,
    key: &str,
) -> Result<(), EncryptSqliteStoreError> {
    let plaintext_options = SqliteConnectOptions::default()
        .filename(database_path)
        .journal_mode(journal_mode)
        .create_if_missing(true);
    let mut conn = try_open(&plaintext_options)
        .await
        .map_err(|source| EncryptSqliteStoreError::Sql {
            ctx: "open plaintext database",
            source,
        })?
        .ok_or(EncryptSqliteStoreError::NotADatabase)?;

    let tmp_path = database_path.with_added_extension("encrypted");
    tokio::fs::remove_file(&tmp_path).await.or_else(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            Ok(())
        } else {
            Err(EncryptSqliteStoreError::RemoveDatabaseFile {
                database_path: tmp_path.clone(),
                source,
            })
        }
    })?;

    let user_version: i64 = sqlx::query_scalar("PRAGMA user_version;")
        .fetch_one(&mut conn)
        .await
        .map_err(|source| EncryptSqliteStoreError::Sql {
            ctx: "fetch user_version",
            source,
        })?;

    sqlx::query("ATTACH DATABASE ? AS encrypted KEY ?;")
        .bind(tmp_path.to_string_lossy())
        .bind(key)
        .execute(&mut conn)
        .await
        .map_err(|source| EncryptSqliteStoreError::Sql {
            ctx: "attach encrypted database",
            source,
        })?;
    conn.execute("SELECT sqlcipher_export('encrypted');")
        .await
        .map_err(|source| EncryptSqliteStoreError::Sql {
            ctx: "sqlcipher_export",
            source,
        })?;

    // sqlcipher_export does not copy user_version, so carry it over explicitly.
    // SAFETY: PRAGMA does not accept bound parameters, but an integer cannot inject SQL.
    conn.execute(AssertSqlSafe(format!(
        "PRAGMA encrypted.user_version = {user_version};"
    )))
    .await
    .map_err(|source| EncryptSqliteStoreError::Sql {
        ctx: "migrate user_version",
        source,
    })?;

    // Attached databases do not inherit WAL mode, so the encrypted copy uses a rollback journal
    // that is deleted on commit. Nothing besides the main file needs to be moved after DETACH.
    conn.execute("DETACH DATABASE encrypted;")
        .await
        .map_err(|source| EncryptSqliteStoreError::Sql {
            ctx: "detach encrypted database",
            source,
        })?;

    conn.close()
        .await
        .map_err(|source| EncryptSqliteStoreError::Sql {
            ctx: "close plain database",
            source,
        })?;

    // Delete journals of plaintext database
    remove_journal_files(database_path).await?;

    // Replace plaintext database with encrypted copy
    tokio::fs::rename(&tmp_path, database_path)
        .await
        .map_err(|source| EncryptSqliteStoreError::MoveDatabaseFile {
            from: tmp_path.clone(),
            to: database_path.to_owned(),
            source,
        })?;

    Ok(())
}

async fn remove_journal_files(database_path: &Path) -> Result<(), EncryptSqliteStoreError> {
    for suffix in SQLITE_JOURNAL_SUFFIXES {
        let journal_path = append_suffix(database_path, suffix);

        tokio::fs::remove_file(&journal_path)
            .await
            .or_else(|source| {
                if source.kind() == std::io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(EncryptSqliteStoreError::RemoveDatabaseFile {
                        database_path: journal_path,
                        source,
                    })
                }
            })?;
    }
    Ok(())
}

fn append_suffix(db_path: &Path, suffix: &str) -> PathBuf {
    let mut file_path = db_path.as_os_str().to_owned();
    file_path.push(suffix);
    PathBuf::from(file_path)
}

/// Opens a connection and reads the schema to force SQLite to decode the first page.
///
/// Returns `Ok(None)` if SQLite reports SQLITE_NOTADB, i.e. the file cannot be read with the
/// given options.
async fn try_open(options: &SqliteConnectOptions) -> sqlx::Result<Option<SqliteConnection>> {
    let mut conn = match SqliteConnection::connect_with(options).await {
        Ok(conn) => conn,
        Err(e) if is_not_a_database(&e) => return Ok(None),
        Err(e) => return Err(e),
    };
    match sqlx::query("SELECT count(*) FROM sqlite_master;")
        .fetch_one(&mut conn)
        .await
    {
        Ok(_) => Ok(Some(conn)),
        Err(e) if is_not_a_database(&e) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Quotes the key as SQL string literal, because pragma values are not bound as parameters.
fn quote_key(key: &str) -> String {
    format!("'{}'", key.replace('\'', "''"))
}

/// SQLCipher reports SQLITE_NOTADB (26, "file is not a database") when a database is opened
/// without the correct key.
fn is_not_a_database(e: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = e {
        db_err.code().is_some_and(|code| code == "26")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "correct horse battery staple";
    const OTHER_KEY: &str = "wrong key";
    const USER_VERSION: i64 = 7;

    #[tokio::test]
    async fn missing_database_is_not_migrated() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.sqlite");

        let result = encrypt_sqlite_store(&path, SqliteJournalMode::Wal, KEY)
            .await
            .unwrap();

        assert!(matches!(result, MigrationResult::NoDatabaseExist));
        assert!(!tokio::fs::try_exists(&path).await.unwrap());
    }

    #[tokio::test]
    async fn encrypts_plaintext_database() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.sqlite");
        create_plaintext_database(&path, SqliteJournalMode::Delete).await;

        let result = encrypt_sqlite_store(&path, SqliteJournalMode::Delete, KEY)
            .await
            .unwrap();

        assert!(matches!(result, MigrationResult::Completed));
        assert_encrypted_with(&path, KEY).await;
        assert_no_leftover_files(&path).await;
    }

    #[tokio::test]
    async fn encrypts_plaintext_database_in_wal_mode() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.sqlite");
        create_plaintext_database(&path, SqliteJournalMode::Wal).await;

        let result = encrypt_sqlite_store(&path, SqliteJournalMode::Wal, KEY)
            .await
            .unwrap();

        assert!(matches!(result, MigrationResult::Completed));
        assert_encrypted_with(&path, KEY).await;
        assert_no_leftover_files(&path).await;
    }

    #[tokio::test]
    async fn removes_stale_journal_files() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.sqlite");
        create_plaintext_database(&path, SqliteJournalMode::Delete).await;
        for suffix in ["-wal", "-shm"] {
            tokio::fs::write(append_suffix(&path, suffix), b"")
                .await
                .unwrap();
        }

        let result = encrypt_sqlite_store(&path, SqliteJournalMode::Delete, KEY)
            .await
            .unwrap();

        assert!(matches!(result, MigrationResult::Completed));
        assert_encrypted_with(&path, KEY).await;
        assert_no_leftover_files(&path).await;
    }

    #[tokio::test]
    async fn replaces_stale_temporary_database() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.sqlite");
        create_plaintext_database(&path, SqliteJournalMode::Delete).await;
        tokio::fs::write(path.with_added_extension("encrypted"), b"stale")
            .await
            .unwrap();

        let result = encrypt_sqlite_store(&path, SqliteJournalMode::Delete, KEY)
            .await
            .unwrap();

        assert!(matches!(result, MigrationResult::Completed));
        assert_encrypted_with(&path, KEY).await;
        assert_no_leftover_files(&path).await;
    }

    #[tokio::test]
    async fn already_encrypted_database_is_left_untouched() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.sqlite");
        create_plaintext_database(&path, SqliteJournalMode::Delete).await;
        encrypt_sqlite_store(&path, SqliteJournalMode::Delete, KEY)
            .await
            .unwrap();
        let encrypted_contents = tokio::fs::read(&path).await.unwrap();

        let result = encrypt_sqlite_store(&path, SqliteJournalMode::Delete, KEY)
            .await
            .unwrap();

        assert!(matches!(result, MigrationResult::AlreadyEncrypted));
        assert_eq!(tokio::fs::read(&path).await.unwrap(), encrypted_contents);
        assert_encrypted_with(&path, KEY).await;
    }

    #[tokio::test]
    async fn database_encrypted_with_different_key_is_rejected() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.sqlite");
        create_plaintext_database(&path, SqliteJournalMode::Delete).await;
        encrypt_sqlite_store(&path, SqliteJournalMode::Delete, KEY)
            .await
            .unwrap();
        let encrypted_contents = tokio::fs::read(&path).await.unwrap();

        let result = encrypt_sqlite_store(&path, SqliteJournalMode::Delete, OTHER_KEY).await;

        assert!(matches!(result, Err(EncryptSqliteStoreError::NotADatabase)));
        assert_eq!(tokio::fs::read(&path).await.unwrap(), encrypted_contents);
        assert_encrypted_with(&path, KEY).await;
    }

    #[tokio::test]
    async fn file_that_is_not_a_database_is_rejected() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("store.sqlite");
        let contents = b"this is definitely not an sqlite database".repeat(64);
        tokio::fs::write(&path, &contents).await.unwrap();

        let result = encrypt_sqlite_store(&path, SqliteJournalMode::Delete, KEY).await;

        assert!(matches!(result, Err(EncryptSqliteStoreError::NotADatabase)));
        assert_eq!(tokio::fs::read(&path).await.unwrap(), contents);
    }

    async fn create_plaintext_database(path: &Path, journal_mode: SqliteJournalMode) {
        let options = SqliteConnectOptions::default()
            .filename(path)
            .journal_mode(journal_mode)
            .create_if_missing(true);
        let mut conn = SqliteConnection::connect_with(&options).await.unwrap();

        conn.execute("CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL);")
            .await
            .unwrap();
        conn.execute("INSERT INTO items (name) VALUES ('first'), ('second');")
            .await
            .unwrap();
        conn.execute(AssertSqlSafe(format!(
            "PRAGMA user_version = {USER_VERSION};"
        )))
        .await
        .unwrap();
        conn.close().await.unwrap();
    }

    async fn open(path: &Path, key: Option<&str>) -> sqlx::Result<SqliteConnection> {
        let mut options = SqliteConnectOptions::default()
            .filename(path)
            .journal_mode(SqliteJournalMode::Delete);
        if let Some(key) = key {
            options = options.pragma("key", quote_key(key));
        }
        SqliteConnection::connect_with(&options).await
    }

    async fn read_items(conn: &mut SqliteConnection) -> sqlx::Result<Vec<String>> {
        sqlx::query_scalar("SELECT name FROM items ORDER BY id;")
            .fetch_all(conn)
            .await
    }

    async fn assert_encrypted_with(path: &Path, key: &str) {
        let mut conn = open(path, Some(key)).await.unwrap();
        assert_eq!(read_items(&mut conn).await.unwrap(), ["first", "second"]);

        let user_version: i64 = sqlx::query_scalar("PRAGMA user_version;")
            .fetch_one(&mut conn)
            .await
            .unwrap();
        assert_eq!(user_version, USER_VERSION);
        conn.close().await.unwrap();

        let plaintext_read = match open(path, None).await {
            Ok(mut conn) => read_items(&mut conn).await.map(|_| ()),
            Err(err) => Err(err),
        };
        assert!(
            plaintext_read.as_ref().is_err_and(is_not_a_database),
            "database must not be readable without a key: {plaintext_read:?}"
        );
    }

    async fn assert_no_leftover_files(path: &Path) {
        let leftovers = SQLITE_JOURNAL_SUFFIXES
            .iter()
            .map(|suffix| append_suffix(path, suffix))
            .chain([path.with_added_extension("encrypted")]);

        for leftover in leftovers {
            assert!(
                !tokio::fs::try_exists(&leftover).await.unwrap(),
                "unexpected leftover file: {}",
                leftover.display()
            );
        }
    }
}
