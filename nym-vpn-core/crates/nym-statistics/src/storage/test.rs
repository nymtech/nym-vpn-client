use nym_sqlx_pool_guard::SqlitePoolGuard;

use crate::storage::{StatsStorage, sqlite::SqliteStatsStorageManager};

pub(crate) async fn mock_database() -> StatsStorage {
    let connection_pool = SqlitePoolGuard::new(
        sqlx::sqlite::SqlitePoolOptions::new()
            .connect(":memory:")
            .await
            .expect("cannot connect to db"),
    );

    sqlx::migrate!("./migrations")
        .run(&*connection_pool)
        .await
        .expect("Failed to run migrations");

    StatsStorage {
        storage_manager: SqliteStatsStorageManager::new(connection_pool),
    }
}
