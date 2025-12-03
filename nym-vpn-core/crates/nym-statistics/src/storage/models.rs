use nym_statistics_common::report::vpn_client::SessionReport as VpnSessionReport;
use time::Date;

#[derive(sqlx::FromRow, PartialEq, Eq, Debug)]
pub(crate) struct SessionReport {
    pub day_utc: Date,
    pub connection_time_ms: i32,
    pub retry_attempt: i32,
    pub session_duration_min: i32,
    pub disconnection_time_ms: i32,
    pub tunnel_type: String,
    pub exit_id: String,
    pub exit_cc: Option<String>,
    pub follow_up_id: Option<String>,
    pub error: Option<String>,
}

#[derive(sqlx::FromRow, PartialEq, Eq, Debug)]
pub(crate) struct SessionReportWithId {
    pub id: i32,
    #[sqlx(flatten)]
    pub report: SessionReport,
}

impl From<SessionReport> for VpnSessionReport {
    fn from(value: SessionReport) -> Self {
        Self {
            start_day_utc: value.day_utc,
            connection_time_ms: value.connection_time_ms,
            retry_attempt: value.retry_attempt,
            session_duration_min: value.session_duration_min,
            disconnection_time_ms: value.disconnection_time_ms,
            tunnel_type: value.tunnel_type,
            exit_id: value.exit_id,
            exit_cc: value.exit_cc,
            follow_up_id: value.follow_up_id,
            error: value.error,
        }
    }
}

impl From<SessionReportWithId> for VpnSessionReport {
    fn from(value: SessionReportWithId) -> Self {
        value.report.into()
    }
}
