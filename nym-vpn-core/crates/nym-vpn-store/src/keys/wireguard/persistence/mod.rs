// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use rand::Rng;
use time::{Duration, OffsetDateTime};

use crate::keys::wireguard::WireguardKeys;

pub(crate) mod ephemeral;
pub(crate) mod on_disk;

const MIN_TTL: Duration = Duration::weeks(1);
const MAX_TTL: Duration = Duration::weeks(2);

fn random_keys() -> WireguardKeys {
    let mut rng = rand::rngs::OsRng;
    WireguardKeys::generate_new(&mut rng)
}

fn random_timestamp_from(start: OffsetDateTime) -> OffsetDateTime {
    let mut rng = rand::rngs::OsRng;
    let random_seconds = rng.gen_range(MIN_TTL.whole_seconds()..MAX_TTL.whole_seconds());
    start.saturating_add(Duration::seconds(random_seconds))
}

fn is_expired(expiration_time: OffsetDateTime) -> bool {
    OffsetDateTime::now_utc() >= expiration_time
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_date_ttls_between_1_to_2_weeks() {
        let now = OffsetDateTime::now_utc();
        let generated = random_timestamp_from(now);
        let diference = generated - now;
        assert_eq!(diference.whole_weeks(), 1);
    }

    #[test]
    fn expiration() {
        let now = OffsetDateTime::now_utc();
        let yesterday = now - Duration::days(1);
        let tomorrow = now + Duration::days(1);

        assert!(is_expired(yesterday));
        assert!(!is_expired(tomorrow));
    }
}
