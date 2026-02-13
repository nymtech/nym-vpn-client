use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct MixnetTrafficConfig {
    pub poisson_parameter_for_loop_cover_stream: Option<u32>,
    pub average_packet_delay: Option<u32>,
    pub message_sending_average_delay: Option<u32>,
    pub disable_poisson_rate: bool,
    pub disable_background_cover_traffic: bool,
    pub min_mixnode_performance: Option<u8>,
    pub min_gateway_mixnet_performance: Option<u8>,
}

macro_rules! impl_mixnet_traffic_config_conversion {
    ($from:ty => $to:ty) => {
        impl From<$from> for $to {
            fn from(value: $from) -> Self {
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
    };
}

impl_mixnet_traffic_config_conversion!(lib::MixnetTrafficConfig => MixnetTrafficConfig);
impl_mixnet_traffic_config_conversion!(MixnetTrafficConfig => lib::MixnetTrafficConfig);

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct MixingDelay {
    pub min_value: u32,
    pub max_value: u32,
    pub default_value: u32,
}

impl From<lib::MixingDelay> for MixingDelay {
    fn from(v: lib::MixingDelay) -> Self {
        Self {
            min_value: v.min_value,
            max_value: v.max_value,
            default_value: v.default_value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct BackgroundCoverTrafficRate {
    pub value: u32,
    pub multiplier: String,
}

impl From<lib::BackgroundCoverTrafficRate> for BackgroundCoverTrafficRate {
    fn from(v: lib::BackgroundCoverTrafficRate) -> Self {
        Self {
            value: v.value(),
            multiplier: v.multiplier(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct ContinuousTrafficSendingRate {
    pub value: u32,
    pub throughput: String,
}

impl From<lib::ContinuousTrafficSendingRate> for ContinuousTrafficSendingRate {
    fn from(v: lib::ContinuousTrafficSendingRate) -> Self {
        Self {
            value: v.value(),
            throughput: v.throughput(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct MixnetTrafficDefaults {
    pub mixing_delay: MixingDelay,
    pub disable_poisson_rate: bool,
    pub default_background_traffic: BackgroundCoverTrafficRate,
    pub default_continuous_traffic: ContinuousTrafficSendingRate,
    pub all_background_traffic: Vec<BackgroundCoverTrafficRate>,
    pub all_continuous_traffic: Vec<ContinuousTrafficSendingRate>,
}

impl MixnetTrafficDefaults {
    pub fn get() -> Self {
        let defaults = lib::MixnetTrafficDefaults;
        Self {
            mixing_delay: defaults.default_mixing_delay().into(),
            disable_poisson_rate: defaults.default_disable_poission_rate(),
            default_background_traffic: defaults.default_background_traffic().into(),
            default_continuous_traffic: defaults.default_continuous_traffic().into(),
            all_background_traffic: defaults
                .all_background_traffic()
                .into_iter()
                .map(Into::into)
                .collect(),
            all_continuous_traffic: defaults
                .all_continuous_traffic()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}
