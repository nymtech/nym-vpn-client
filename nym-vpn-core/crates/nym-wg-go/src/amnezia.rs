//! Interface to amneziawg-go (fork of wireguard-go) allowing optional use of Amnezia features.
//!

use crate::UapiConfigBuilder;

use rand::{Rng, RngCore};

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
};

const BASE: AmneziaConfig = AmneziaConfig {
    junk_pkt_count: 4,
    junk_pkt_min_size: 40,
    junk_pkt_max_size: 70,
    init_pkt_junk_size: 0,
    response_pkt_junk_size: 0,
    init_pkt_magic_header: 1,
    response_pkt_magic_header: 2,
    under_load_pkt_magic_header: 3,
    transport_pkt_magic_header: 4,
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
    pub fn rand(rng: &mut impl RngCore) -> Self {
        for _ in 0..16 {
            let c = Self {
                junk_pkt_count: rng.gen_range(3..10),
                junk_pkt_min_size: rng.gen_range(0..900),
                junk_pkt_max_size: 1000,
                init_pkt_junk_size: rng.gen_range(15..150),
                response_pkt_junk_size: rng.gen_range(15..150),
                init_pkt_magic_header: rng.gen_range(5..i32::MAX),
                response_pkt_magic_header: rng.gen_range(5..i32::MAX),
                under_load_pkt_magic_header: rng.gen_range(5..i32::MAX),
                transport_pkt_magic_header: rng.gen_range(5..i32::MAX),
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
        config_builder.add("h3", self.under_load_pkt_magic_header.to_string().as_str());
        config_builder.add("h4", self.transport_pkt_magic_header.to_string().as_str());
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
        if self.junk_pkt_count > 128
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
            .contains(&self.transport_pkt_magic_header)
        {
            return false;
        }
        true
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
        };
        let mut config_builder = UapiConfigBuilder::new();
        c.append_to(&mut config_builder);
        assert_eq!(
            config_builder.into_bytes(),
            b"jc=1\njmin=20\njmax=30\ns1=40\ns2=50\nh1=11\nh2=12\nh3=13\nh4=14\n\n"
        );
    }
}
