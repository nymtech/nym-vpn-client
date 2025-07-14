use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum StatsStorageError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migrate error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("file permissions error for {path:?}: {source}")]
    FilePermissions {
        path: PathBuf,
        source: std::io::Error,
    },
}
