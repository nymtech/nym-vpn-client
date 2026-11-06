//! Interface to amneziawg-go (fork of wireguard-go) allowing optional use of Amnezia features.
//!

use crate::UapiConfigBuilder;

use rand::{Rng, RngExt};

const OFF: AmneziaConfig = AmneziaConfig {
    junk_pkt_count: 0,
    junk_pkt_min_size: 0,
    junk_pkt_max_size: 0,
    init_pkt_junk_size: 0,
    response_pkt_junk_size: 0,
    init_pkt_magic_header: 1,
    response_pkt_magic_header: 2,
    under_load_pkt_magic_header: 3,
    transport_pkt_magic_header: 4,
    cookie_pkt_junk_size: 0,
    transport_pkt_junk_size: 0,
    under_load_pkt_magic_header_max: None,
    transport_pkt_magic_header_max: None,
    header_protection_key: None,
    random_trailers: false,
};

const BASE: AmneziaConfig = AmneziaConfig {
    junk_pkt_count: 4,
    junk_pkt_min_size: 40,
    junk_pkt_max_size: 70,
    ..OFF
};

/// Hold Amnezia-wireguard configuration parameters.
///
/// All parameters should be the same between Client and Server, except Jc - it can vary.
///
/// - Jc — 1 ≤ Jc ≤ 128; recommended range is from 3 to 10 inclusive
/// - Jmin — Jmin < Jmax; recommended value is 50
/// - Jmax — Jmin < Jmax ≤ 1280; recommended value is 1000
/// - S1 — S1 < 1280; S1 + 56 ≠ S2; recommended range is from 15 to 150 inclusive
/// - S2 — S2 < 1280; recommended range is from 15 to 150 inclusive
/// - H1/H2/H3/H4 — must be unique among each other;
///   recommended range is from 5 to 2_147_483_647  (2^31 - 1   i.e. signed 32 bit int) inclusive
///
/// Note: changes to S1, S2, H1, H2, H3, and H4 are required to match between client
/// and server. The connection will not work otherwise.
#[derive(Debug, Clone, PartialEq)]
pub struct AmneziaConfig {
    /// Jc - Count of junk packets to send BEFORE sending the handshake Init message.
    pub junk_pkt_count: u8, // Jc
    /// Jmin - Minimum size in bytes of the Junk packets enabled by `junk_pkt_count`
    pub junk_pkt_min_size: u16, // Jmin
    /// Jmax - Maximum size in bytes of the Junk packets enabled by `junk_pkt_count`
    pub junk_pkt_max_size: u16, // Jmax
    /// S1 - Numer of byte to PREPEND to the Handshake init message
    pub init_pkt_junk_size: u16, // S1
    /// S2 - Number of bytes to PREPEND to the Handshake response message
    pub response_pkt_junk_size: u16, // S2
    /// H1 - Re-map handshake Init packet header type indicator to this value
    pub init_pkt_magic_header: i32, // H1
    /// H2 - Re-map handshake reponse packet header type indicator to this value
    pub response_pkt_magic_header: i32, // H2
    /// H3 - Re-map under load packet header type indicator to this value
    pub under_load_pkt_magic_header: i32, // H3
    /// H4 - Re-map transport packet header type indicator to this value
    pub transport_pkt_magic_header: i32, // H4
    /// S3 - Number of bytes to PREPEND to the cookie reply message (AmneziaWG 3.1)
    pub cookie_pkt_junk_size: u16, // S3
    /// S4 - Number of bytes to PREPEND to every transport message (AmneziaWG 3.1)
    pub transport_pkt_junk_size: u16, // S4
    /// Upper end of the H3 range, the single H3 value when `None` (AmneziaWG 3.1)
    pub under_load_pkt_magic_header_max: Option<i32>,
    /// Upper end of the H4 range, the single H4 value when `None` (AmneziaWG 3.1)
    pub transport_pkt_magic_header_max: Option<i32>,
    /// Hex encoded 32 byte key masking every header; needs S1-S4 >= 12 (AmneziaWG 3.1)
    pub header_protection_key: Option<String>,
    /// Random trailing bytes on handshake and cookie messages (AmneziaWG 3.1)
    pub random_trailers: bool,
}

impl Default for AmneziaConfig {
    fn default() -> Self {
        OFF.clone()
    }
}

impl AmneziaConfig {
    /// Disabled Amnezia Configuration
    pub const OFF: Self = OFF;
    /// Enables only the minimum Amnezia features, while ensuring compatibility with plain
    /// wireguard peers.
    pub const BASE: Self = BASE;

    /// Creates a randomized configuration with parameters within suggested ranges.
    ///
    /// Attempts to retry if there is a collision in [H1, H2, H3, H4]. This should
    /// almost never happen given the range available (5 to i32::MAX) unless the provided
    /// rng is bad. If the rng is bad, then amneziawg will break anyways so we panic.
    pub fn rand(rng: &mut impl Rng) -> Self {
        for _ in 0..16 {
            let c = Self {
                junk_pkt_count: rng.random_range(3..10),
                junk_pkt_min_size: rng.random_range(0..900),
                junk_pkt_max_size: 1000,
                init_pkt_junk_size: rng.random_range(15..150),
                response_pkt_junk_size: rng.random_range(15..150),
                init_pkt_magic_header: rng.random_range(5..i32::MAX),
                response_pkt_magic_header: rng.random_range(5..i32::MAX),
                under_load_pkt_magic_header: rng.random_range(5..i32::MAX),
                transport_pkt_magic_header: rng.random_range(5..i32::MAX),
                ..OFF
            };
            if c.validate() {
                return c;
            }
        }
        panic!("this should not be possible");
    }

    /// Adds the contained AmneziaWG parameters to the UAPI Config
    pub fn append_to(&self, config_builder: &mut UapiConfigBuilder) {
        if self == &OFF {
            return;
        }
        config_builder.add("jc", self.junk_pkt_count.to_string().as_str());
        config_builder.add("jmin", self.junk_pkt_min_size.to_string().as_str());
        config_builder.add("jmax", self.junk_pkt_max_size.to_string().as_str());

        if self == &BASE {
            return;
        }

        config_builder.add("s1", self.init_pkt_junk_size.to_string().as_str());
        config_builder.add("s2", self.response_pkt_junk_size.to_string().as_str());
        config_builder.add("h1", self.init_pkt_magic_header.to_string().as_str());
        config_builder.add("h2", self.response_pkt_magic_header.to_string().as_str());
        config_builder.add(
            "h3",
            header_range(
                self.under_load_pkt_magic_header,
                self.under_load_pkt_magic_header_max,
            )
            .as_str(),
        );
        config_builder.add(
            "h4",
            header_range(
                self.transport_pkt_magic_header,
                self.transport_pkt_magic_header_max,
            )
            .as_str(),
        );
        if self.cookie_pkt_junk_size > 0 {
            config_builder.add("s3", self.cookie_pkt_junk_size.to_string().as_str());
        }
        if self.transport_pkt_junk_size > 0 {
            config_builder.add("s4", self.transport_pkt_junk_size.to_string().as_str());
        }
        if let Some(key) = &self.header_protection_key {
            config_builder.add("header_protection_key", key.as_str());
        }
        if self.random_trailers {
            config_builder.add("random_trailers", "true");
        }
    }

    /// Check if the provided configuration is valid
    ///
    /// - Jc — 1 ≤ Jc ≤ 128; recommended range is from 3 to 10 inclusive
    /// - Jmin — Jmin < Jmax; recommended value is 50
    /// - Jmax — Jmin < Jmax ≤ 1280; recommended value is 1000
    /// - S1 — S1 < 1280; S1 + 56 ≠ S2; recommended range is from 15 to 150 inclusive
    /// - S2 — S2 < 1280; recommended range is from 15 to 150 inclusive
    /// - H1/H2/H3/H4 — must be unique among each other;
    ///   recommended range is from 5 to 2_147_483_647  (2^31 - 1   i.e. signed 32 bit int) inclusive
    pub fn validate(&self) -> bool {
        // Header protection key and padding rules are enforced by the bridge params and again by
        // amneziawg-go; only the H3/H4 ranges need to stay ordered and apart here.
        let h3 = self.under_load_pkt_magic_header;
        let h4 = self.transport_pkt_magic_header;
        let h3_max = self.under_load_pkt_magic_header_max.unwrap_or(h3);
        let h4_max = self.transport_pkt_magic_header_max.unwrap_or(h4);
        h3 <= h3_max
            && h4 <= h4_max
            && (h3_max < h4 || h4_max < h3)
            && !(self.junk_pkt_count > 128
                || self.junk_pkt_max_size > 1280
                || self.junk_pkt_min_size > self.junk_pkt_max_size
                || self.init_pkt_junk_size > 1280
                || self.response_pkt_junk_size > 1280
                || [
                    self.response_pkt_magic_header,
                    self.under_load_pkt_magic_header,
                    self.transport_pkt_magic_header,
                ]
                .contains(&self.init_pkt_magic_header)
                || [
                    self.init_pkt_magic_header,
                    self.under_load_pkt_magic_header,
                    self.transport_pkt_magic_header,
                ]
                .contains(&self.response_pkt_magic_header)
                || [
                    self.init_pkt_magic_header,
                    self.response_pkt_magic_header,
                    self.transport_pkt_magic_header,
                ]
                .contains(&self.under_load_pkt_magic_header)
                || [
                    self.init_pkt_magic_header,
                    self.response_pkt_magic_header,
                    self.under_load_pkt_magic_header,
                ]
                .contains(&self.transport_pkt_magic_header))
    }
}

/// `h` or `h-max`, the header range format amneziawg-go 3.1 reads.
fn header_range(h: i32, max: Option<i32>) -> String {
    match max {
        Some(max) => format!("{h}-{max}"),
        None => h.to_string(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode_amnezia_config() {
        let mut config_builder = UapiConfigBuilder::new();
        OFF.append_to(&mut config_builder);
        assert_eq!(config_builder.into_bytes(), b"\n");

        let mut config_builder = UapiConfigBuilder::new();
        BASE.append_to(&mut config_builder);
        assert_eq!(config_builder.into_bytes(), b"jc=4\njmin=40\njmax=70\n\n");

        let c = AmneziaConfig {
            junk_pkt_count: 1,
            junk_pkt_min_size: 20,
            junk_pkt_max_size: 30,
            init_pkt_junk_size: 40,
            response_pkt_junk_size: 50,
            init_pkt_magic_header: 11,
            response_pkt_magic_header: 12,
            under_load_pkt_magic_header: 13,
            transport_pkt_magic_header: 14,
            ..OFF
        };
        let mut config_builder = UapiConfigBuilder::new();
        c.append_to(&mut config_builder);
        assert_eq!(
            config_builder.into_bytes(),
            b"jc=1\njmin=20\njmax=30\ns1=40\ns2=50\nh1=11\nh2=12\nh3=13\nh4=14\n\n"
        );
    }

    #[test]
    fn test_encode_amnezia_3_1_config() {
        let c = AmneziaConfig {
            init_pkt_junk_size: 40,
            response_pkt_junk_size: 50,
            under_load_pkt_magic_header: 13,
            transport_pkt_magic_header: 100,
            cookie_pkt_junk_size: 20,
            transport_pkt_junk_size: 16,
            under_load_pkt_magic_header_max: Some(99),
            transport_pkt_magic_header_max: Some(200),
            header_protection_key: Some("ab".repeat(32)),
            random_trailers: true,
            ..BASE
        };
        assert!(c.validate());

        let mut config_builder = UapiConfigBuilder::new();
        c.append_to(&mut config_builder);
        let expected = format!(
            "jc=4\njmin=40\njmax=70\ns1=40\ns2=50\nh1=1\nh2=2\nh3=13-99\nh4=100-200\ns3=20\ns4=16\n\
             header_protection_key={}\nrandom_trailers=true\n\n",
            "ab".repeat(32)
        );
        assert_eq!(config_builder.into_bytes(), expected.as_bytes());
    }

    #[test]
    fn test_validate_amnezia_3_1_config() {
        let valid = AmneziaConfig {
            init_pkt_junk_size: 40,
            response_pkt_junk_size: 50,
            under_load_pkt_magic_header: 13,
            transport_pkt_magic_header: 100,
            cookie_pkt_junk_size: 20,
            transport_pkt_junk_size: 16,
            under_load_pkt_magic_header_max: Some(99),
            transport_pkt_magic_header_max: Some(200),
            header_protection_key: Some("ab".repeat(32)),
            ..BASE
        };
        let overlapping = AmneziaConfig {
            under_load_pkt_magic_header_max: Some(150),
            ..valid.clone()
        };
        assert!(valid.validate());
        assert!(!overlapping.validate());
    }
}
