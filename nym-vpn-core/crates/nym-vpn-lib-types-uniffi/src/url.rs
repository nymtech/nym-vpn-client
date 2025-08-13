// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use url::Url;

uniffi::custom_type!(Url, String, {
    remote,
    try_lift: |val| Ok(Url::from_str(&val)?),
    lower: |val| val.to_string()
});
