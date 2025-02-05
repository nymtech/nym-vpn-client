use anyhow::Result;
use nym_vpn_proto as p;
use time::{Duration, OffsetDateTime};
use tracing::{error, instrument};

#[instrument]
pub fn proto_timestamp_to_datetime(timestamp: p::Timestamp) -> Result<OffsetDateTime> {
    let date_time = OffsetDateTime::from_unix_timestamp(timestamp.seconds)
        .inspect_err(|e| error!("failed to parse timestamp [{}]: {}", timestamp.seconds, e))?;
    // kinda overkill to get nanosec precision >_> but why not
    Ok(date_time + Duration::nanoseconds(timestamp.nanos as i64))
}
