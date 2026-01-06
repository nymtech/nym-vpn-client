// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub(crate) mod v5 {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub struct MixnetTrafficConfig {
        pub poisson_parameter_for_loop_cover_stream: Option<u32>,
        pub average_packet_delay: Option<u32>,
        pub message_sending_average_delay: Option<u32>,

        pub disable_poisson_rate: bool,
        pub disable_background_cover_traffic: bool,

        pub min_mixnode_performance: Option<u8>,
        pub min_gateway_mixnet_performance: Option<u8>,
    }

    impl From<MixnetTrafficConfig> for nym_vpn_lib_types::MixnetTrafficConfig {
        fn from(value: MixnetTrafficConfig) -> Self {
            Self {
                poisson_parameter_for_loop_cover_stream: value
                    .poisson_parameter_for_loop_cover_stream,
                average_packet_delay: value.average_packet_delay,
                message_sending_average_delay: value.message_sending_average_delay,
                disable_poisson_rate: value.disable_poisson_rate,
                disable_background_cover_traffic: value.disable_background_cover_traffic,
                min_mixnode_performance: value.min_mixnode_performance,
                min_gateway_mixnet_performance: value.min_gateway_mixnet_performance,
            }
        }
    }

    // This is only required for the latest version of the external entry point representation
    impl From<&nym_vpn_lib_types::MixnetTrafficConfig> for MixnetTrafficConfig {
        fn from(value: &nym_vpn_lib_types::MixnetTrafficConfig) -> Self {
            Self {
                poisson_parameter_for_loop_cover_stream: value
                    .poisson_parameter_for_loop_cover_stream,
                average_packet_delay: value.average_packet_delay,
                message_sending_average_delay: value.message_sending_average_delay,
                disable_poisson_rate: value.disable_poisson_rate,
                disable_background_cover_traffic: value.disable_background_cover_traffic,
                min_mixnode_performance: value.min_mixnode_performance,
                min_gateway_mixnet_performance: value.min_gateway_mixnet_performance,
            }
        }
    }
}
