use crate::storage::{StatsStorage, sqlite::SqliteStatsStorageManager};

pub(crate) async fn mock_database() -> StatsStorage {
    let conn = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(":memory:")
        .await
        .expect("cannot connect to db");

    sqlx::migrate!("./migrations")
        .run(&conn)
        .await
        .expect("Failed to run migrations");

    StatsStorage {
        storage_manager: SqliteStatsStorageManager::new(conn),
    }
}
