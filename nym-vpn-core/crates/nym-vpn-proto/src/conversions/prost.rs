// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use time::OffsetDateTime;

pub fn prost_timestamp_into_offset_datetime(
    timestamp: prost_types::Timestamp,
) -> Result<OffsetDateTime, time::Error> {
    OffsetDateTime::from_unix_timestamp(timestamp.seconds)
        .map(|t| t + time::Duration::nanoseconds(timestamp.nanos as i64))
        .map_err(time::Error::from)
}

pub fn offset_datetime_into_proto_timestamp(
    datetime: time::OffsetDateTime,
) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: datetime.unix_timestamp(),
        nanos: datetime.nanosecond() as i32,
    }
}
