// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use futures::{StreamExt, stream::BoxStream};
use nym_ipc::AuthenticationMaterial;
use tokio::{
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tonic::{Request, Response, Status, transport::Server};

#[cfg(any(target_os = "macos", target_os = "windows"))]
use nym_vpn_lib_types::SplitApp;
use nym_vpn_lib_types::{
    EnableSocks5Request, EntryPoint, ExitPoint, GetDeeplinkParams, ListGatewaysOptions,
    LookupGatewayFilters, ProfileOptions, TargetState, TunnelEvent,
};

use nym_vpn_proto::proto::{
    self, MixnetTrafficConfig,
    nym_vpn_service_server::{NymVpnService, NymVpnServiceServer},
};

use nym_vpn_lib::service::{SetNetworkError, Socks5Error, VpnServiceCommand};

pub type Result<T> = std::result::Result<T, tonic::Status>;

#[cfg(windows)]
// The Nym serial number of the SSL certificate we use to sign release builds
// in CI.
const NYM_CERTIFICATE_SERIAL_NUMBER: &str = "4ec9356d8c87f9cf3ccf60e7bdad022f";
// The MacOS signing requirement signifying that the binary was signed by apple
// certificate with Nym's identifiers
#[cfg(target_os = "macos")]
const CLIENT_SIGNING_REQUIREMENT: &str = r#"anchor apple generic and certificate leaf[subject.OU] = "VW5DZLFHM5" and (identifier "net.nymtech.vpn" or identifier "net.nymtech.vpn.cli")"#;
#[cfg(target_os = "macos")]
const DAEMON_SIGNING_REQUIREMENT: &str = r#"anchor apple generic and certificate leaf[subject.OU] = "VW5DZLFHM5" and identifier "net.nymtech.vpn.daemon""#;
#[cfg(target_os = "linux")]
const NYM_VPN_GROUP_NAME: &str = "nym-vpn";

pub struct CommandInterface {
    // Send commands to the VPN service
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,

    // Broadcast tunnel events to our API endpoint listeners
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
}

impl CommandInterface {
    fn new(
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
    ) -> Self {
        Self {
            vpn_command_tx,
            tunnel_event_rx,
        }
    }

    async fn send_and_wait<R, F, O>(&self, command: F, opts: O) -> Result<R>
    where
        F: FnOnce(oneshot::Sender<R>, O) -> VpnServiceCommand,
    {
        let (tx, rx) = oneshot::channel();

        self.vpn_command_tx
            .send(command(tx, opts))
            .map_err(|_| tonic::Status::internal("Command channel is closed"))?;

        rx.await
            .map_err(|_| tonic::Status::internal("Response channel is closed"))
    }
}

#[tonic::async_trait]
impl NymVpnService for CommandInterface {
    async fn info(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::InfoResponse>> {
        let info = self.send_and_wait(VpnServiceCommand::Info, ()).await?;
        let response = proto::InfoResponse::from(info);

        Ok(tonic::Response::new(response))
    }

    async fn get_config(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetConfigResponse>> {
        let config = self.send_and_wait(VpnServiceCommand::GetConfig, ()).await?;

        let response = proto::GetConfigResponse {
            config: Some(proto::VpnServiceConfig::from(config)),
        };

        Ok(tonic::Response::new(response))
    }

    async fn set_entry_point(
        &self,
        request: tonic::Request<proto::EntryNode>,
    ) -> Result<tonic::Response<()>> {
        let entry_point = EntryPoint::try_from(request.into_inner())
            .map_err(|e| tonic::Status::invalid_argument(format!("Invalid entry point: {e}")))?;

        let _ = self
            .send_and_wait(VpnServiceCommand::SetEntryPoint, entry_point)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to set VPN entry point: {e}")))?;

        Ok(tonic::Response::new(()))
    }

    async fn set_exit_point(
        &self,
        request: tonic::Request<proto::ExitNode>,
    ) -> Result<tonic::Response<()>> {
        let exit_point = ExitPoint::try_from(request.into_inner())
            .map_err(|e| tonic::Status::invalid_argument(format!("Invalid exit point: {e}")))?;

        let _ = self
            .send_and_wait(VpnServiceCommand::SetExitPoint, exit_point)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to set VPN exit point: {e}")))?;

        Ok(tonic::Response::new(()))
    }

    async fn set_disable_ipv6(&self, request: tonic::Request<bool>) -> Result<tonic::Response<()>> {
        let disable_ipv6 = request.into_inner();

        let _ = self
            .send_and_wait(VpnServiceCommand::SetDisableIPv6, disable_ipv6)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to set IPv6 config: {e}")))?;

        Ok(tonic::Response::new(()))
    }

    async fn set_enable_two_hop(
        &self,
        request: tonic::Request<bool>,
    ) -> Result<tonic::Response<()>> {
        let enable_two_hop = request.into_inner();

        let _ = self
            .send_and_wait(VpnServiceCommand::SetEnableTwoHop, enable_two_hop)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to set two-hop config: {e}")))?;

        Ok(tonic::Response::new(()))
    }

    async fn set_enable_bridges(
        &self,
        request: tonic::Request<bool>,
    ) -> Result<tonic::Response<()>> {
        let enable_bridges = request.into_inner();

        let _ = self
            .send_and_wait(VpnServiceCommand::SetEnableBridges, enable_bridges)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to set enable bridges: {e}")))?;

        Ok(tonic::Response::new(()))
    }

    async fn set_enable_ad_blocking(
        &self,
        request: tonic::Request<bool>,
    ) -> Result<tonic::Response<()>> {
        let enable_ad_blocking = request.into_inner();

        let _ = self
            .send_and_wait(VpnServiceCommand::SetEnableAdBlocking, enable_ad_blocking)
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("Failed to set ad-blocking config: {e}"))
            })?;

        Ok(tonic::Response::new(()))
    }

    async fn set_fronting_mode(
        &self,
        request: tonic::Request<proto::FrontingModeRequest>,
    ) -> Result<tonic::Response<()>> {
        let fronting_mode_request = request.into_inner();
        let fronting_mode = proto::FrontingModes::try_from(fronting_mode_request.mode)
            .map_err(|e| tonic::Status::invalid_argument(format!("Invalid fronting mode: {e}")))?;

        let fronting_mode = fronting_mode.into();

        let _ = self
            .send_and_wait(VpnServiceCommand::SetFrontingMode, fronting_mode)
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("Failed to set fronting mode config: {e}"))
            })?;

        Ok(tonic::Response::new(()))
    }

    async fn set_netstack(&self, request: tonic::Request<bool>) -> Result<tonic::Response<()>> {
        let netstack = request.into_inner();

        let _ = self
            .send_and_wait(VpnServiceCommand::SetNetstack, netstack)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to set netstack config: {e}")))?;

        Ok(tonic::Response::new(()))
    }

    async fn set_allow_lan(&self, request: tonic::Request<bool>) -> Result<tonic::Response<()>> {
        let allow_lan = request.into_inner();

        let _ = self
            .send_and_wait(VpnServiceCommand::SetAllowLan, allow_lan)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to set allow lan: {e}")))?;

        Ok(tonic::Response::new(()))
    }

    async fn set_residential_exit(
        &self,
        request: tonic::Request<bool>,
    ) -> Result<tonic::Response<()>> {
        let residential_exit = request.into_inner();

        let _ = self
            .send_and_wait(VpnServiceCommand::SetResidentialExit, residential_exit)
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("Failed to set residential exit only: {e}"))
            })?;

        Ok(tonic::Response::new(()))
    }

    async fn set_enable_custom_dns(
        &self,
        request: tonic::Request<bool>,
    ) -> Result<tonic::Response<()>> {
        let enable_custom_dns = request.into_inner();

        let _ = self
            .send_and_wait(VpnServiceCommand::SetEnableCustomDns, enable_custom_dns)
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("Failed to set enable custom DNS: {e}"))
            })?;

        Ok(tonic::Response::new(()))
    }

    async fn set_custom_dns(
        &self,
        request: tonic::Request<proto::IpAddrList>,
    ) -> Result<tonic::Response<()>> {
        let custom_dns: Vec<IpAddr> = request
            .into_inner()
            .try_into()
            .map_err(|e| tonic::Status::invalid_argument(format!("Invalid Custom DNS: {e}")))?;

        let _ = self
            .send_and_wait(VpnServiceCommand::SetCustomDns, custom_dns)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to set custom DNS: {e}")))?;

        Ok(tonic::Response::new(()))
    }

    async fn set_mixnet_traffic_config(
        &self,
        request: Request<MixnetTrafficConfig>,
    ) -> std::result::Result<Response<()>, Status> {
        let mixnet_traffic_config: nym_vpn_lib_types::MixnetTrafficConfig =
            request.into_inner().into();

        self.send_and_wait(
            VpnServiceCommand::SetMixnetTrafficConfig,
            mixnet_traffic_config,
        )
        .await
        .map_err(|e| Status::internal(format!("[set_mixnet_traffic_config] transport error: {e}")))?
        .map_err(|err| {
            Status::invalid_argument(format!(
                "[set_mixnet_traffic_config] validation failed: {err}"
            ))
        })?;

        Ok(Response::new(()))
    }

    async fn set_enable_geo_location(
        &self,
        request: tonic::Request<bool>,
    ) -> std::result::Result<Response<()>, Status> {
        let enable_geo_location = request.into_inner();

        self.send_and_wait(VpnServiceCommand::SetEnableGeoLocation, enable_geo_location)
            .await
            .map_err(|e| {
                Status::internal(format!(
                    "[set_enable gateway_selection_algorithm] transport error: {e}"
                ))
            })?
            .map_err(|err| {
                Status::invalid_argument(format!(
                    "[set_enable gateway_selection_algorithm] validation failed: {err}"
                ))
            })?;

        Ok(Response::new(()))
    }

    async fn set_enable_gateway_independence(
        &self,
        request: tonic::Request<bool>,
    ) -> std::result::Result<Response<()>, Status> {
        let enable_gateway_independence = request.into_inner();

        self.send_and_wait(
            VpnServiceCommand::SetEnableGatewayIndependence,
            enable_gateway_independence,
        )
        .await
        .map_err(|e| {
            Status::internal(format!(
                "[set_enable gateway_independence] transport error: {e}"
            ))
        })?;

        Ok(Response::new(()))
    }

    async fn set_gateway_independence_notifications(
        &self,
        request: tonic::Request<bool>,
    ) -> Result<Response<()>> {
        let enable_notifications = request.into_inner();

        self.send_and_wait(
            VpnServiceCommand::SetGatewayIndependenceNotifications,
            enable_notifications,
        )
        .await
        .map_err(|e| {
            Status::internal(format!(
                "[set gateway_independence_notifications] transport error: {e}"
            ))
        })?;

        Ok(Response::new(()))
    }

    async fn set_geo_exclusion_enabled(
        &self,
        request: tonic::Request<bool>,
    ) -> Result<tonic::Response<()>> {
        let enabled = request.into_inner();
        self.send_and_wait(VpnServiceCommand::SetGeoExclusionEnabled, enabled)
            .await?;
        Ok(tonic::Response::new(()))
    }

    async fn set_geo_exclusion_listen_port(
        &self,
        request: tonic::Request<proto::GeoExclusionListenPortRequest>,
    ) -> Result<tonic::Response<()>> {
        let port = request.into_inner().listen_port;
        if port == 0 || port > u16::MAX as u32 {
            return Err(tonic::Status::invalid_argument(format!(
                "listen_port must be a valid non-zero port number (got {port})"
            )));
        }
        self.send_and_wait(VpnServiceCommand::SetGeoExclusionListenPort, port as u16)
            .await?
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;
        Ok(tonic::Response::new(()))
    }

    async fn set_geo_exclusion_excluded_countries(
        &self,
        request: tonic::Request<proto::StringList>,
    ) -> Result<tonic::Response<()>> {
        let countries = request.into_inner().values;

        self.send_and_wait(
            VpnServiceCommand::SetGeoExclusionExcludedCountries,
            countries,
        )
        .await?
        .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;
        Ok(tonic::Response::new(()))
    }

    async fn get_recent_gateways(
        &self,
        request: tonic::Request<proto::GetRecentGatewaysParams>,
    ) -> Result<tonic::Response<proto::RecentGateways>> {
        let tunnel_type =
            nym_vpn_lib_types::GetRecentGatewaysParams::try_from(request.into_inner())
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Invalid recent gateway params: {e}"))
                })?
                .tunnel_type;
        let response = self
            .send_and_wait(VpnServiceCommand::GetRecentGateways, tunnel_type)
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get recent gateways: {err}"))
            })?;

        Ok(tonic::Response::new(response.into()))
    }

    async fn set_network(&self, request: tonic::Request<String>) -> Result<tonic::Response<()>> {
        let network = request.into_inner();
        let status = self
            .send_and_wait(VpnServiceCommand::SetNetwork, network)
            .await?;

        status.map_err(|e| match e {
            SetNetworkError::NetworkNotFound(network_name) => {
                tonic::Status::not_found(format!("Network not found: {network_name}"))
            }
            e => tonic::Status::internal(e.to_string()),
        })?;

        Ok(tonic::Response::new(()))
    }

    async fn get_system_messages(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetSystemMessagesResponse>> {
        let messages = self
            .send_and_wait(VpnServiceCommand::GetSystemMessages, ())
            .await?;

        let messages = messages
            .into_iter()
            .map(proto::SystemMessage::from)
            .collect::<Vec<_>>();
        let response = proto::GetSystemMessagesResponse { messages };

        Ok(tonic::Response::new(response))
    }

    async fn get_network_compatibility(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetNetworkCompatibilityResponse>> {
        let network_compatibility = self
            .send_and_wait(VpnServiceCommand::GetNetworkCompatibility, ())
            .await?
            .map(proto::NetworkCompatibility::from);

        let response = proto::GetNetworkCompatibilityResponse {
            network_compatibility,
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_feature_flags(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetFeatureFlagsResponse>> {
        let feature_flags = self
            .send_and_wait(VpnServiceCommand::GetFeatureFlags, ())
            .await?
            .ok_or(tonic::Status::not_found("Feature flags not found"))?;

        Ok(tonic::Response::new(feature_flags.into()))
    }

    async fn get_default_dns(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::IpAddrList>> {
        let dns_ips = self
            .send_and_wait(VpnServiceCommand::GetDefaultDns, ())
            .await?;
        let ipaddr_list = proto::IpAddrList::from(dns_ips);
        Ok(tonic::Response::new(ipaddr_list))
    }

    async fn connect_tunnel(&self, _request: tonic::Request<()>) -> Result<tonic::Response<bool>> {
        let accepted = self
            .send_and_wait(VpnServiceCommand::SetTargetState, TargetState::Secured)
            .await?;

        Ok(tonic::Response::new(accepted))
    }

    async fn reconnect_tunnel(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<bool>> {
        let accepted = self.send_and_wait(VpnServiceCommand::Reconnect, ()).await?;

        Ok(tonic::Response::new(accepted))
    }

    async fn disconnect_tunnel(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<bool>> {
        let accepted = self
            .send_and_wait(VpnServiceCommand::SetTargetState, TargetState::Unsecured)
            .await?;

        Ok(tonic::Response::new(accepted))
    }

    async fn get_tunnel_state(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::TunnelState>> {
        let tunnel_state = self
            .send_and_wait(VpnServiceCommand::GetTunnelState, ())
            .await
            .map(proto::TunnelState::from)?;

        Ok(tonic::Response::new(tunnel_state))
    }

    type ListenToEventsStream = BoxStream<'static, Result<proto::TunnelEvent>>;
    async fn listen_to_events(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::ListenToEventsStream>> {
        let rx = self.tunnel_event_rx.resubscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(rx).map(|event| {
            event
                .map(proto::TunnelEvent::from)
                .map_err(|_| tonic::Status::internal("Failed to receive tunnel event"))
        });
        Ok(tonic::Response::new(
            Box::pin(stream) as Self::ListenToEventsStream
        ))
    }

    async fn list_gateways(
        &self,
        request: tonic::Request<proto::ListGatewaysRequest>,
    ) -> Result<tonic::Response<proto::ListGatewaysResponse>> {
        let options = ListGatewaysOptions::try_from(request.into_inner())
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

        let gateways = self
            .send_and_wait(VpnServiceCommand::ListGateways, options)
            .await?
            .map_err(|err| tonic::Status::internal(format!("Failed to list gateways: {err}")))?;

        let response = proto::ListGatewaysResponse {
            gateways: gateways
                .into_iter()
                .map(proto::GatewayResponse::from)
                .collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn list_filtered_gateways(
        &self,
        request: tonic::Request<proto::LookupGatewayFilters>,
    ) -> Result<tonic::Response<proto::ListGatewaysResponse>> {
        let filters = LookupGatewayFilters::try_from(request.into_inner())
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

        let gateways = self
            .send_and_wait(VpnServiceCommand::ListFilteredGateways, filters)
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to list filtered gateways: {err}"))
            })?;

        let response = proto::ListGatewaysResponse {
            gateways: gateways
                .into_iter()
                .map(proto::GatewayResponse::from)
                .collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn store_account(
        &self,
        request: tonic::Request<proto::StoreAccountRequest>,
    ) -> Result<tonic::Response<proto::AccountCommandResponse>> {
        let store_request = nym_vpn_lib_types::StoreAccountRequest::try_from(request.into_inner())
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

        let result = self
            .send_and_wait(VpnServiceCommand::StoreAccount, store_request)
            .await?;

        let response = proto::AccountCommandResponse {
            error: result.err().map(proto::AccountCommandError::from),
        };

        Ok(tonic::Response::new(response))
    }

    async fn account_balance(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::AccountBalanceResponse>> {
        let response = self
            .send_and_wait(VpnServiceCommand::DecentralisedBalance, ())
            .await?;

        let response = proto::AccountBalanceResponse::from(response);

        Ok(tonic::Response::new(response))
    }

    async fn decentralised_obtain_ticketbooks(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::AccountCommandResponse>> {
        let result = self
            .send_and_wait(VpnServiceCommand::DecentralisedObtainTicketbooks, ())
            .await?;

        let response = proto::AccountCommandResponse {
            error: result.err().map(proto::AccountCommandError::from),
        };

        Ok(tonic::Response::new(response))
    }

    async fn is_account_stored(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<bool>> {
        let is_stored = self
            .send_and_wait(VpnServiceCommand::IsAccountStored, ())
            .await?;

        Ok(tonic::Response::new(is_stored))
    }

    async fn get_account_mode(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetAccountModeResponse>> {
        let mode = self
            .send_and_wait(VpnServiceCommand::GetAccountMode, ())
            .await?;

        Ok(tonic::Response::new(mode.into()))
    }

    async fn forget_account(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::AccountCommandResponse>> {
        let result = self
            .send_and_wait(VpnServiceCommand::ForgetAccount, ())
            .await?;

        let response = proto::AccountCommandResponse {
            error: result.err().map(proto::AccountCommandError::from),
        };

        Ok(tonic::Response::new(response))
    }

    async fn rotate_keys(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::AccountCommandResponse>> {
        let result = self
            .send_and_wait(VpnServiceCommand::RotateKeys, ())
            .await?;

        let response = proto::AccountCommandResponse {
            error: result.err().map(proto::AccountCommandError::from),
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_account_identity(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetAccountIdentityResponse>> {
        let account_identity = self
            .send_and_wait(VpnServiceCommand::GetAccountIdentity, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get account identity: {err:?}"))
            })?;

        Ok(tonic::Response::new(proto::GetAccountIdentityResponse {
            account_identity,
        }))
    }

    async fn get_canonical_account_identity(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetAccountIdentityResponse>> {
        let account_identity = self
            .send_and_wait(VpnServiceCommand::GetCanonicalAccountIdentity, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!(
                    "Failed to get canonical account identity: {err:?}"
                ))
            })?;

        Ok(tonic::Response::new(proto::GetAccountIdentityResponse {
            account_identity,
        }))
    }

    async fn get_account_links(
        &self,
        request: tonic::Request<proto::GetAccountLinksRequest>,
    ) -> Result<tonic::Response<proto::AccountManagement>> {
        let locale = request.into_inner().locale;

        let account_links = self
            .send_and_wait(VpnServiceCommand::GetAccountLinks, locale)
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get account links: {err}"))
            })?;

        Ok(tonic::Response::new(proto::AccountManagement::from(
            account_links,
        )))
    }

    async fn get_account_state(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::AccountControllerState>> {
        let account_controller_state = self
            .send_and_wait(VpnServiceCommand::GetAccountState, ())
            .await
            .map(proto::AccountControllerState::from)?;

        Ok(tonic::Response::new(account_controller_state))
    }

    async fn refresh_account_state(
        &self,
        request: tonic::Request<proto::RefreshAccountStateRequest>,
    ) -> Result<tonic::Response<()>> {
        let force = request.into_inner().force;
        self.send_and_wait(VpnServiceCommand::RefreshAccountState, force)
            .await?;

        Ok(tonic::Response::new(()))
    }

    async fn get_account_usage(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetAccountUsageResponse>> {
        let account_usage = self
            .send_and_wait(VpnServiceCommand::GetAccountUsage, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get account usage: {err}"))
            })?;

        Ok(tonic::Response::new(proto::GetAccountUsageResponse {
            account_usages: Some(proto::get_account_usage_response::AccountUsages::from(
                account_usage,
            )),
        }))
    }

    async fn reset_device_identity(
        &self,
        request: tonic::Request<proto::ResetDeviceIdentityRequest>,
    ) -> Result<tonic::Response<()>> {
        let seed: Option<[u8; 32]> = request
            .into_inner()
            .seed
            .map(|seed| {
                seed.as_slice()
                    .try_into()
                    .map_err(|_| tonic::Status::invalid_argument("Seed must be 32 bytes long"))
            })
            .transpose()?;

        self.send_and_wait(VpnServiceCommand::ResetDeviceIdentity, seed)
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to reset device identity: {err}"))
            })?;

        Ok(tonic::Response::new(()))
    }

    async fn get_device_identity(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetDeviceIdentityResponse>> {
        let device_identity = self
            .send_and_wait(VpnServiceCommand::GetDeviceIdentity, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get device identity: {err}"))
            })?;

        Ok(tonic::Response::new(proto::GetDeviceIdentityResponse {
            device_identity,
        }))
    }

    async fn get_devices(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetDevicesResponse>> {
        let devices = self
            .send_and_wait(VpnServiceCommand::GetDevices, ())
            .await?
            .map_err(|err| tonic::Status::internal(format!("Failed to get devices: {err}")))?;

        Ok(tonic::Response::new(proto::GetDevicesResponse {
            devices: Some(proto::get_devices_response::Devices::from(devices)),
        }))
    }

    async fn get_active_devices(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetDevicesResponse>> {
        let devices = self
            .send_and_wait(VpnServiceCommand::GetActiveDevices, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get active devices: {err}"))
            })?;

        Ok(tonic::Response::new(proto::GetDevicesResponse {
            devices: Some(proto::get_devices_response::Devices::from(devices)),
        }))
    }

    async fn get_available_tickets(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::AvailableTickets>> {
        let available_ticketbooks = self
            .send_and_wait(VpnServiceCommand::GetAvailableTickets, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get available tickets: {err}"))
            })?;

        let available_tickets = nym_vpn_lib_types::AvailableTickets::from(available_ticketbooks);
        let response = proto::AvailableTickets::from(available_tickets);

        Ok(tonic::Response::new(response))
    }

    async fn restock_ticketbooks(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<()>> {
        self.send_and_wait(VpnServiceCommand::RestockTicketbooks, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to restock ticketbooks: {err}"))
            })?;

        Ok(tonic::Response::new(()))
    }

    async fn get_account_summary(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::VpnAccountSummaryResponse>> {
        let account_summary = self
            .send_and_wait(VpnServiceCommand::GetAccountSummary, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get account summary: {err}"))
            })?;

        let response = proto::VpnAccountSummaryResponse {
            account_summary: account_summary.map(proto::VpnAccountSummary::from),
        };

        Ok(tonic::Response::new(response))
    }

    async fn handle_subscription_payment(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<()>> {
        self.send_and_wait(VpnServiceCommand::HandleSubscriptionPayment, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to handle subscription payment: {err}"))
            })?;

        Ok(tonic::Response::new(()))
    }

    async fn get_deeplink(
        &self,
        request: tonic::Request<proto::GetDeeplinkParams>,
    ) -> Result<tonic::Response<String>> {
        let req = request.into_inner();

        let params: GetDeeplinkParams = req.try_into().map_err(|err| {
            tonic::Status::invalid_argument(format!("Invalid get deeplink request: {err}"))
        })?;

        let url = self
            .send_and_wait(VpnServiceCommand::GetDeeplink, params)
            .await?
            .map_err(|err| tonic::Status::internal(format!("Failed to get deeplink: {err}")))?;

        Ok(tonic::Response::new(url.to_string()))
    }

    async fn get_autologin_deeplink(
        &self,
        request: tonic::Request<proto::GetDeeplinkParams>,
    ) -> Result<tonic::Response<proto::AutologinResponse>> {
        let req = request.into_inner();

        let params: GetDeeplinkParams = req.try_into().map_err(|err| {
            tonic::Status::invalid_argument(format!(
                "Invalid get autologin deeplink request: {err}"
            ))
        })?;

        let url = self
            .send_and_wait(VpnServiceCommand::GetAutologinDeeplink, params)
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get autologin deeplink: {err}"))
            })?;

        Ok(tonic::Response::new(url.into()))
    }

    async fn deeplink_store_account(
        &self,
        request: tonic::Request<String>,
    ) -> Result<tonic::Response<proto::AccountCommandResponse>> {
        let deeplink_callback_url = request.into_inner();

        let result = self
            .send_and_wait(
                VpnServiceCommand::DeeplinkStoreAccount,
                deeplink_callback_url,
            )
            .await?;

        let response = proto::AccountCommandResponse {
            error: result.err().map(proto::AccountCommandError::from),
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_log_path(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetLogPathResponse>> {
        let log_path = self
            .send_and_wait(VpnServiceCommand::GetLogPath, ())
            .await?
            .map(proto::LogPath::try_from)
            .transpose()
            .map_err(|err| tonic::Status::internal(format!("Failed to get log path: {err}")))?;

        Ok(tonic::Response::new(proto::GetLogPathResponse { log_path }))
    }

    async fn delete_log_file(&self, _request: tonic::Request<()>) -> Result<tonic::Response<()>> {
        self.send_and_wait(VpnServiceCommand::DeleteLogFile, ())
            .await?;

        Ok(tonic::Response::new(()))
    }

    async fn is_sentry_enabled(&self, _: tonic::Request<()>) -> Result<tonic::Response<bool>> {
        let result = self
            .send_and_wait(VpnServiceCommand::IsSentryEnabled, ())
            .await?;
        Ok(tonic::Response::new(result))
    }

    async fn enable_sentry(&self, _: tonic::Request<()>) -> Result<tonic::Response<()>> {
        self.send_and_wait(VpnServiceCommand::ToggleSentry, true)
            .await?
            .map_err(|err| {
                tracing::error!("Failed to enable sentry monitoring: {err}");
                tonic::Status::internal("failed to enable sentry")
            })?;
        Ok(tonic::Response::new(()))
    }

    async fn disable_sentry(&self, _: tonic::Request<()>) -> Result<tonic::Response<()>> {
        self.send_and_wait(VpnServiceCommand::ToggleSentry, false)
            .await?
            .map_err(|err| {
                tracing::error!("Failed to disable sentry monitoring: {err}");
                tonic::Status::internal("failed to disable sentry")
            })?;
        Ok(tonic::Response::new(()))
    }

    async fn network_stats_set_enabled(
        &self,
        request: tonic::Request<bool>,
    ) -> Result<tonic::Response<()>> {
        let enabled = request.into_inner();

        let _ = self
            .send_and_wait(VpnServiceCommand::EnableNetStats, enabled)
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("Failed to enable/disable network statistics: {e}"))
            })?;

        Ok(tonic::Response::new(()))
    }

    async fn network_stats_allow_disconnected(
        &self,
        request: tonic::Request<bool>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let allow_disconnected = request.into_inner();

        let _ = self
            .send_and_wait(
                VpnServiceCommand::AllowDisconnectedNetStats,
                allow_disconnected,
            )
            .await
            .map_err(|e| {
                tonic::Status::internal(format!(
                    "Failed to set network statistics allow_disconnected: {e}"
                ))
            })?;

        Ok(tonic::Response::new(()))
    }

    async fn network_stats_reset_seed(
        &self,
        request: tonic::Request<proto::NetworkStatsResetSeedRequest>,
    ) -> Result<tonic::Response<()>> {
        let seed = request.into_inner().seed;

        let _ = self
            .send_and_wait(VpnServiceCommand::ResetNetStatsSeed, seed)
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("Failed to reset network statistics seed: {e}"))
            })?;

        Ok(tonic::Response::new(()))
    }

    async fn network_stats_get_seed(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::NetworkStatisticsIdentity>> {
        let identity = self
            .send_and_wait(VpnServiceCommand::GetNetStatsSeed, ())
            .await?
            .map_err(|e| {
                tonic::Status::internal(format!("Failed to get network statistics identity: {e}"))
            })?;

        Ok(tonic::Response::new(identity.into()))
    }

    async fn enable_socks5(
        &self,
        request: tonic::Request<proto::EnableSocks5Request>,
    ) -> Result<tonic::Response<()>> {
        let req = request.into_inner();

        let enable_socks5_request: EnableSocks5Request = req.try_into().map_err(|e| {
            tonic::Status::invalid_argument(format!("Invalid Enable SOCKS5 Request: {e}"))
        })?;

        self.send_and_wait(VpnServiceCommand::EnableSocks5, enable_socks5_request)
            .await?
            .map_err(|err| {
                tracing::error!("Failed to enable SOCKS5 proxy: {err}");
                match err {
                    Socks5Error::GatewayNotSupported => tonic::Status::failed_precondition(
                        "Gateway does not support SOCKS5 network requester",
                    ),
                    Socks5Error::InvalidConfig(msg) => tonic::Status::failed_precondition(msg),
                    Socks5Error::LazySocks5Error(_) => {
                        tonic::Status::internal(format!("Failed to enable SOCKS5 proxy: {err}"))
                    }
                }
            })?;

        Ok(tonic::Response::new(()))
    }

    async fn disable_socks5(&self, _: tonic::Request<()>) -> Result<tonic::Response<()>> {
        self.send_and_wait(VpnServiceCommand::DisableSocks5, ())
            .await?
            .map_err(|err| {
                tracing::error!("Failed to disable SOCKS5 proxy: {err}");
                tonic::Status::internal(format!("Failed to disable SOCKS5 proxy: {err}"))
            })?;

        Ok(tonic::Response::new(()))
    }

    async fn get_socks5_status(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::Socks5Status>> {
        let status = self
            .send_and_wait(VpnServiceCommand::GetSocks5Status, ())
            .await?
            .map_err(|err| {
                tracing::debug!("Failed to get SOCKS5 status: {err}");
                tonic::Status::internal(format!("Failed to get SOCKS5 status: {err}"))
            })?;

        // Convert from lib type to proto type using From trait
        let proto_status = proto::Socks5Status::from(status);

        Ok(tonic::Response::new(proto_status))
    }

    async fn get_privy_derivation_message(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::PrivyDerivationMessage>> {
        Ok(tonic::Response::new(proto::PrivyDerivationMessage {
            message: nym_vpn_lib::privy::message_to_sign(),
        }))
    }

    async fn run_diagnostic(
        &self,
        request: tonic::Request<proto::DiagnosticRunParams>,
    ) -> Result<tonic::Response<proto::DiagnosticReport>> {
        let req = request.into_inner();
        let report = self
            .send_and_wait(VpnServiceCommand::RunDiagnostic, req.into())
            .await?;

        let proto_report = report.try_into().map_err(|e| {
            tonic::Status::internal(format!("Failed to run diagnostic report: {e}"))
        })?;

        Ok(tonic::Response::new(proto_report))
    }

    async fn register_diagnostic(
        &self,
        request: tonic::Request<proto::DiagnosticRegisterParams>,
    ) -> Result<tonic::Response<proto::RegistrationReport>> {
        let req = request.into_inner();
        let register_params = req.try_into().map_err(|e| {
            tonic::Status::invalid_argument(format!("Invalid Register diagnostic argument: {e}"))
        })?;
        let report = self
            .send_and_wait(VpnServiceCommand::RegisterDiagnostic, register_params)
            .await?;

        let proto_report = report.try_into().map_err(|e| {
            tonic::Status::internal(format!("Failed to run diagnostic report: {e}"))
        })?;

        Ok(tonic::Response::new(proto_report))
    }

    async fn get_tentative_gateways(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::TentativeGateways>> {
        let tentative_gateways = self
            .send_and_wait(VpnServiceCommand::GetTentativeGateways, ())
            .await?;

        Ok(tonic::Response::new(tentative_gateways.into()))
    }

    async fn is_split_tunnel_supported(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<bool>> {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            let is_available = self
                .send_and_wait(
                    VpnServiceCommand::IsSplitTunnelSupported,
                    _request.into_inner(),
                )
                .await?;
            Ok(tonic::Response::new(is_available))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        Err(tonic::Status::internal("Unsupported platform"))
    }

    async fn set_enable_split_tunnel(
        &self,
        _request: tonic::Request<bool>,
    ) -> Result<tonic::Response<()>> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            self.send_and_wait(
                VpnServiceCommand::SetEnableSplitTunnel,
                _request.into_inner(),
            )
            .await?;
            Ok(tonic::Response::new(()))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        Err(tonic::Status::internal("Unsupported platform"))
    }

    async fn add_split_tunnel_app(
        &self,
        _request: tonic::Request<proto::SplitApp>,
    ) -> Result<tonic::Response<()>> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            let app = SplitApp::from(_request.into_inner());
            self.send_and_wait(VpnServiceCommand::AddSplitTunnelApp, app)
                .await?;
            Ok(tonic::Response::new(()))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        Err(tonic::Status::internal("Unsupported platform"))
    }

    async fn remove_split_tunnel_app(
        &self,
        _request: tonic::Request<proto::SplitApp>,
    ) -> Result<tonic::Response<()>> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            let app = SplitApp::from(_request.into_inner());
            self.send_and_wait(VpnServiceCommand::RemoveSplitTunnelApp, app)
                .await?;
            Ok(tonic::Response::new(()))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        Err(tonic::Status::internal("Unsupported platform"))
    }

    async fn clear_split_tunnel_apps(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<()>> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            self.send_and_wait(VpnServiceCommand::ClearSplitTunnelApps, ())
                .await?;
            Ok(tonic::Response::new(()))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        Err(tonic::Status::internal("Unsupported platform"))
    }

    async fn get_split_tunnel_excluded_processes(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::SplitTunnelExcludedProcessList>> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let res = self
                .send_and_wait(VpnServiceCommand::GetSplitTunnelExcludedProcesses, ())
                .await
                .map(proto::SplitTunnelExcludedProcessList::from)?;
            Ok(tonic::Response::new(res))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        Err(tonic::Status::internal("Unsupported platform"))
    }

    async fn need_full_disk_permissions(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<bool>> {
        #[cfg(target_os = "macos")]
        {
            let need_fda = self
                .send_and_wait(VpnServiceCommand::NeedFullDiskPermissions, ())
                .await?;
            Ok(tonic::Response::new(need_fda))
        }

        #[cfg(not(target_os = "macos"))]
        Err(tonic::Status::internal("Unsupported platform"))
    }

    async fn add_split_tunnel_process(
        &self,
        _request: tonic::Request<i32>,
    ) -> Result<tonic::Response<()>> {
        #[cfg(target_os = "linux")]
        {
            let pid = _request.into_inner();
            self.send_and_wait(VpnServiceCommand::AddSplitTunnelProcess, pid)
                .await?;
            Ok(tonic::Response::new(()))
        }

        #[cfg(not(target_os = "linux"))]
        Err(tonic::Status::internal("Unsupported platform"))
    }

    async fn remove_split_tunnel_process(
        &self,
        _request: tonic::Request<i32>,
    ) -> Result<tonic::Response<()>> {
        #[cfg(target_os = "linux")]
        {
            let pid = _request.into_inner();
            self.send_and_wait(VpnServiceCommand::RemoveSplitTunnelProcess, pid)
                .await?;
            Ok(tonic::Response::new(()))
        }

        #[cfg(not(target_os = "linux"))]
        Err(tonic::Status::internal("Unsupported platform"))
    }

    async fn clear_split_tunnel_processes(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<()>> {
        #[cfg(target_os = "linux")]
        {
            self.send_and_wait(VpnServiceCommand::ClearSplitTunnelProcesses, ())
                .await?;
            Ok(tonic::Response::new(()))
        }

        #[cfg(not(target_os = "linux"))]
        Err(tonic::Status::internal("Unsupported platform"))
    }

    async fn set_profile(
        &self,
        request: tonic::Request<proto::ProfileOptions>,
    ) -> Result<tonic::Response<()>> {
        let profile_options = ProfileOptions::try_from(request.into_inner())
            .map_err(|e| tonic::Status::invalid_argument(format!("Invalid profile: {e}")))?;

        let _ = self
            .send_and_wait(VpnServiceCommand::SetProfile, profile_options.profile)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to set profile: {e}")))?;

        Ok(tonic::Response::new(()))
    }
}

pub async fn start_command_interface(
    disable_client_verification: bool,
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
    shutdown_token: CancellationToken,
) -> std::io::Result<(JoinHandle<()>, UnboundedReceiver<VpnServiceCommand>)> {
    tracing::debug!("Starting command interface");

    let (vpn_command_tx, vpn_command_rx) = mpsc::unbounded_channel();

    // Wrap the unix socket or named pipe into a stream that can be used by tonic
    let incoming = nym_ipc::server::create_incoming(
        default_socket_path(),
        AuthenticationMaterial::new(
            disable_client_verification,
            #[cfg(target_os = "windows")]
            NYM_CERTIFICATE_SERIAL_NUMBER.to_string(),
            #[cfg(target_os = "macos")]
            nym_ipc::SigningRequirements {
                daemon_req: DAEMON_SIGNING_REQUIREMENT.to_string(),
                client_req: CLIENT_SIGNING_REQUIREMENT.to_string(),
            },
            #[cfg(target_os = "linux")]
            NYM_VPN_GROUP_NAME,
            #[cfg(unix)]
            shutdown_token.child_token(),
        ),
    )
    .await?;

    let server_handle = tokio::spawn(async move {
        let socket_listener_handle = tokio::spawn(async move {
            let command_interface = CommandInterface::new(vpn_command_tx, tunnel_event_rx);

            let server = Server::builder().add_service(NymVpnServiceServer::new(command_interface));
            // Linux needs to handle the shutdown internally first, as it spawns an authentication prompt that needs to
            // be closed in case of shutdown, so the stream can't be shutdown by tonic before that happens...
            #[cfg(target_os = "linux")]
            let ret = server.serve_with_incoming(incoming).await;
            // ... but non-Linux desktops do authentication by signature verification, which can be stopped at any moment,
            // so we serve it by attaching the shutdown token to tonic directly
            #[cfg(not(target_os = "linux"))]
            let ret = server
                .serve_with_incoming_shutdown(
                    incoming,
                    shutdown_token.child_token().cancelled_owned(),
                )
                .await;
            match ret {
                Ok(()) => {
                    tracing::info!("Socket listener has finished");
                }
                Err(e) => {
                    tracing::error!("Socket listener exited with error: {}", e);
                }
            }
        });

        if let Err(e) = socket_listener_handle.await {
            tracing::error!("Failed to join on socket listener: {}", e);
        }

        tracing::info!("Command interface exiting");
    });

    Ok((server_handle, vpn_command_rx))
}

fn default_socket_path() -> std::path::PathBuf {
    #[cfg(unix)]
    {
        std::path::PathBuf::from("/var/run/nym-vpn.sock")
    }

    #[cfg(windows)]
    {
        std::path::PathBuf::from(r"\\.\pipe\nym-vpn")
    }
}
