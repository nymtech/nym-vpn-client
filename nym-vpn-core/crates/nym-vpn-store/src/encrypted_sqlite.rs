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
        .pragma("key", key.to_owned());

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
    let mut file_path = db_path.to_owned();
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
mod tests {}
