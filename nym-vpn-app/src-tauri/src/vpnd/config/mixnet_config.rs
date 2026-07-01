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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixnet_traffic_config_round_trips_through_the_lib_type() {
        let app = MixnetTrafficConfig {
            poisson_parameter_for_loop_cover_stream: Some(5),
            average_packet_delay: Some(50),
            message_sending_average_delay: Some(20),
            disable_poisson_rate: true,
            disable_background_cover_traffic: false,
            min_mixnode_performance: Some(80),
            min_gateway_mixnet_performance: Some(70),
        };
        let lib_cfg: lib::MixnetTrafficConfig = app.clone().into();
        let back: MixnetTrafficConfig = lib_cfg.into();

        assert_eq!(
            back.poisson_parameter_for_loop_cover_stream,
            app.poisson_parameter_for_loop_cover_stream
        );
        assert_eq!(back.average_packet_delay, app.average_packet_delay);
        assert_eq!(
            back.message_sending_average_delay,
            app.message_sending_average_delay
        );
        assert_eq!(back.disable_poisson_rate, app.disable_poisson_rate);
        assert_eq!(
            back.disable_background_cover_traffic,
            app.disable_background_cover_traffic
        );
        assert_eq!(back.min_mixnode_performance, app.min_mixnode_performance);
        assert_eq!(
            back.min_gateway_mixnet_performance,
            app.min_gateway_mixnet_performance
        );
    }

    #[test]
    fn defaults_are_well_formed() {
        // Exercises MixingDelay / BackgroundCoverTrafficRate /
        // ContinuousTrafficSendingRate conversions transitively.
        let d = MixnetTrafficDefaults::get();

        assert!(d.mixing_delay.min_value <= d.mixing_delay.default_value);
        assert!(d.mixing_delay.default_value <= d.mixing_delay.max_value);
        assert!(!d.all_background_traffic.is_empty());
        assert!(!d.all_continuous_traffic.is_empty());
        assert!(!d.default_background_traffic.multiplier.is_empty());
        assert!(!d.default_continuous_traffic.throughput.is_empty());
    }
}
