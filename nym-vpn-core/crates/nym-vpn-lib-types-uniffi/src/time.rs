// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use time::OffsetDateTime;

uniffi::custom_type!(OffsetDateTime, i64, {
    remote,
    try_lift: |val| Ok(OffsetDateTime::from_unix_timestamp(val)?),
    lower: |val| val.unix_timestamp()
});
