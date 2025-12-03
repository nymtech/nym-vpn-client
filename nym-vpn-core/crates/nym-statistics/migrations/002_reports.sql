CREATE TABLE pending_session_report (
    id                      INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    day_utc                 DATE NOT NULL,
    connection_time_ms      INTEGER NOT NULL, 
    retry_attempt           INTEGER NOT NULL,
    session_duration_min    INTEGER NOT NULL,
    disconnection_time_ms   INTEGER NOT NULL, 
    tunnel_type             TEXT NOT NULL,
    exit_id                 TEXT NOT NULL,
    exit_cc                 TEXT,
    follow_up_id            TEXT,
    error                   TEXT
);