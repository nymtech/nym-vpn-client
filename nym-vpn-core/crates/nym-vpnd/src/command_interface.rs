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

use nym_vpn_lib_types::{
    AccountBalanceResponse, AccountCommandResponse, AccountControllerState, AppCompatibility,
    AutologinResponse, AvailableTickets, BridgeMode, DiagnosticRegisterParams, DiagnosticReport,
    DiagnosticRunParams, EnableSocks5Request, EntryPoint, ExitPoint, FeatureFlags, FrontingMode,
    Gateway, GetDeeplinkParams, GetRecentGatewaysParams, ListGatewaysOptions, LogPath,
    LookupGatewayFilters, MixnetTrafficConfig, NetworkStatisticsIdentity, NymVpnDevice,
    NymVpnUsage, ParsedAccountLinks, PrivyDerivationMessage, ProfileOptions, RecentGateways,
    RegistrationReport, Socks5Status, SplitApp, SplitTunnelExcludedProcessList,
    StoreAccountRequest, StoredAccountMode, SystemMessage, TargetState, TentativeGateways,
    TunnelEvent, TunnelState, VpnAccountSummary, VpnServiceConfig, VpnServiceInfo,
};
use nym_vpn_rpc::{NymVpnService, NymVpnServiceServer};

use nym_vpn_lib::service::{SetNetworkError, Socks5Error, VpnServiceCommand};

pub type Result<T> = std::result::Result<T, Status>;

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
            .map_err(|_| Status::internal("Command channel is closed"))?;

        rx.await
            .map_err(|_| Status::internal("Response channel is closed"))
    }
}

#[tonic::async_trait]
impl NymVpnService for CommandInterface {
    async fn info(&self, _request: Request<()>) -> Result<Response<VpnServiceInfo>> {
        let info = self.send_and_wait(VpnServiceCommand::Info, ()).await?;

        Ok(Response::new(info))
    }

    async fn get_config(&self, _request: Request<()>) -> Result<Response<VpnServiceConfig>> {
        let config = self.send_and_wait(VpnServiceCommand::GetConfig, ()).await?;

        Ok(Response::new(config))
    }

    async fn set_entry_point(&self, request: Request<EntryPoint>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetEntryPoint, request.into_inner())
            .await
            .map_err(|e| Status::internal(format!("Failed to set VPN entry point: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_exit_point(&self, request: Request<ExitPoint>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetExitPoint, request.into_inner())
            .await
            .map_err(|e| Status::internal(format!("Failed to set VPN exit point: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_disable_ipv6(&self, request: Request<bool>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetDisableIPv6, request.into_inner())
            .await
            .map_err(|e| Status::internal(format!("Failed to set IPv6 config: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_enable_two_hop(&self, request: Request<bool>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetEnableTwoHop, request.into_inner())
            .await
            .map_err(|e| Status::internal(format!("Failed to set two-hop config: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_bridge_mode(&self, request: Request<BridgeMode>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetBridgeMode, request.into_inner())
            .await
            .map_err(|e| Status::internal(format!("Failed to set bridge mode: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_enable_ad_blocking(&self, request: Request<bool>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetEnableAdBlocking, request.into_inner())
            .await
            .map_err(|e| Status::internal(format!("Failed to set ad-blocking config: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_enable_conflict_detection(&self, request: Request<bool>) -> Result<Response<()>> {
        self.send_and_wait(
            VpnServiceCommand::SetEnableConflictDetection,
            request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to set conflict-detection config: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_fronting_mode(&self, request: Request<FrontingMode>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetFrontingMode, request.into_inner())
            .await
            .map_err(|e| Status::internal(format!("Failed to set fronting mode config: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_netstack(&self, request: Request<bool>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetNetstack, request.into_inner())
            .await
            .map_err(|e| Status::internal(format!("Failed to set netstack config: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_allow_lan(&self, request: Request<bool>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetAllowLan, request.into_inner())
            .await
            .map_err(|e| Status::internal(format!("Failed to set allow lan: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_residential_exit(&self, request: Request<bool>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetResidentialExit, request.into_inner())
            .await
            .map_err(|e| Status::internal(format!("Failed to set residential exit only: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_enable_custom_dns(&self, request: Request<bool>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetEnableCustomDns, request.into_inner())
            .await
            .map_err(|e| Status::internal(format!("Failed to set enable custom DNS: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_custom_dns(&self, request: Request<Vec<IpAddr>>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetCustomDns, request.into_inner())
            .await
            .map_err(|e| Status::internal(format!("Failed to set custom DNS: {e}")))?;

        Ok(Response::new(()))
    }

    async fn set_mixnet_traffic_config(
        &self,
        request: Request<MixnetTrafficConfig>,
    ) -> Result<Response<()>> {
        self.send_and_wait(
            VpnServiceCommand::SetMixnetTrafficConfig,
            request.into_inner(),
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

    async fn set_enable_geo_location(&self, request: Request<bool>) -> Result<Response<()>> {
        self.send_and_wait(
            VpnServiceCommand::SetEnableGeoLocation,
            request.into_inner(),
        )
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
        request: Request<bool>,
    ) -> Result<Response<()>> {
        self.send_and_wait(
            VpnServiceCommand::SetEnableGatewayIndependence,
            request.into_inner(),
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
        request: Request<bool>,
    ) -> Result<Response<()>> {
        self.send_and_wait(
            VpnServiceCommand::SetGatewayIndependenceNotifications,
            request.into_inner(),
        )
        .await
        .map_err(|e| {
            Status::internal(format!(
                "[set gateway_independence_notifications] transport error: {e}"
            ))
        })?;

        Ok(Response::new(()))
    }

    async fn set_geo_exclusion_enabled(&self, request: Request<bool>) -> Result<Response<()>> {
        self.send_and_wait(
            VpnServiceCommand::SetGeoExclusionEnabled,
            request.into_inner(),
        )
        .await?;

        Ok(Response::new(()))
    }

    async fn set_geo_exclusion_listen_port(&self, request: Request<u16>) -> Result<Response<()>> {
        let port = request.into_inner();
        if port == 0 {
            return Err(Status::invalid_argument(
                "listen_port must be a valid non-zero port number (got 0)",
            ));
        }

        self.send_and_wait(VpnServiceCommand::SetGeoExclusionListenPort, port)
            .await?
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        Ok(Response::new(()))
    }

    async fn set_geo_exclusion_excluded_countries(
        &self,
        request: Request<Vec<String>>,
    ) -> Result<Response<()>> {
        self.send_and_wait(
            VpnServiceCommand::SetGeoExclusionExcludedCountries,
            request.into_inner(),
        )
        .await?
        .map_err(|err| Status::invalid_argument(err.to_string()))?;

        Ok(Response::new(()))
    }

    async fn get_recent_gateways(
        &self,
        request: Request<GetRecentGatewaysParams>,
    ) -> Result<Response<RecentGateways>> {
        let recent_gateways = self
            .send_and_wait(
                VpnServiceCommand::GetRecentGateways,
                request.into_inner().tunnel_type,
            )
            .await?
            .map_err(|err| Status::internal(format!("Failed to get recent gateways: {err}")))?;

        Ok(Response::new(recent_gateways))
    }

    async fn set_network(&self, request: Request<String>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetNetwork, request.into_inner())
            .await?
            .map_err(|e| match e {
                SetNetworkError::NetworkNotFound(network_name) => {
                    Status::not_found(format!("Network not found: {network_name}"))
                }
                e => Status::internal(e.to_string()),
            })?;

        Ok(Response::new(()))
    }

    async fn get_system_messages(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Vec<SystemMessage>>> {
        let messages = self
            .send_and_wait(VpnServiceCommand::GetSystemMessages, ())
            .await?;

        Ok(Response::new(messages))
    }

    async fn get_app_compatibility(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Option<AppCompatibility>>> {
        let compatibility = self
            .send_and_wait(VpnServiceCommand::GetAppCompatibility, ())
            .await?;

        Ok(Response::new(compatibility))
    }

    async fn get_feature_flags(&self, _request: Request<()>) -> Result<Response<FeatureFlags>> {
        let feature_flags = self
            .send_and_wait(VpnServiceCommand::GetFeatureFlags, ())
            .await?
            .ok_or(Status::not_found("Feature flags not found"))?;

        Ok(Response::new(feature_flags))
    }

    async fn get_default_dns(&self, _request: Request<()>) -> Result<Response<Vec<IpAddr>>> {
        let dns_ips = self
            .send_and_wait(VpnServiceCommand::GetDefaultDns, ())
            .await?;

        Ok(Response::new(dns_ips))
    }

    async fn connect_tunnel(&self, _request: Request<()>) -> Result<Response<bool>> {
        let accepted = self
            .send_and_wait(VpnServiceCommand::SetTargetState, TargetState::Secured)
            .await?;

        Ok(Response::new(accepted))
    }

    async fn reconnect_tunnel(&self, _request: Request<()>) -> Result<Response<bool>> {
        let accepted = self.send_and_wait(VpnServiceCommand::Reconnect, ()).await?;

        Ok(Response::new(accepted))
    }

    async fn disconnect_tunnel(&self, _request: Request<()>) -> Result<Response<bool>> {
        let accepted = self
            .send_and_wait(VpnServiceCommand::SetTargetState, TargetState::Unsecured)
            .await?;

        Ok(Response::new(accepted))
    }

    async fn get_tunnel_state(&self, _request: Request<()>) -> Result<Response<TunnelState>> {
        let tunnel_state = self
            .send_and_wait(VpnServiceCommand::GetTunnelState, ())
            .await?;

        Ok(Response::new(tunnel_state))
    }

    type ListenToEventsStream = BoxStream<'static, Result<TunnelEvent>>;

    async fn listen_to_events(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::ListenToEventsStream>> {
        let rx = self.tunnel_event_rx.resubscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .map(|event| event.map_err(|_| Status::internal("Failed to receive tunnel event")));

        Ok(Response::new(Box::pin(stream) as Self::ListenToEventsStream))
    }

    async fn list_gateways(
        &self,
        request: Request<ListGatewaysOptions>,
    ) -> Result<Response<Vec<Gateway>>> {
        let gateways = self
            .send_and_wait(VpnServiceCommand::ListGateways, request.into_inner())
            .await?
            .map_err(|err| Status::internal(format!("Failed to list gateways: {err}")))?;

        Ok(Response::new(gateways))
    }

    async fn list_filtered_gateways(
        &self,
        request: Request<LookupGatewayFilters>,
    ) -> Result<Response<Vec<Gateway>>> {
        let gateways = self
            .send_and_wait(
                VpnServiceCommand::ListFilteredGateways,
                request.into_inner(),
            )
            .await?
            .map_err(|err| Status::internal(format!("Failed to list filtered gateways: {err}")))?;

        Ok(Response::new(gateways))
    }

    async fn store_account(
        &self,
        request: Request<StoreAccountRequest>,
    ) -> Result<Response<AccountCommandResponse>> {
        let result = self
            .send_and_wait(VpnServiceCommand::StoreAccount, request.into_inner())
            .await?;

        Ok(Response::new(AccountCommandResponse {
            error: result.err(),
        }))
    }

    async fn account_balance(
        &self,
        _request: Request<()>,
    ) -> Result<Response<AccountBalanceResponse>> {
        let response = self
            .send_and_wait(VpnServiceCommand::DecentralisedBalance, ())
            .await?;

        Ok(Response::new(response))
    }

    async fn decentralised_obtain_ticketbooks(
        &self,
        _request: Request<()>,
    ) -> Result<Response<AccountCommandResponse>> {
        let result = self
            .send_and_wait(VpnServiceCommand::DecentralisedObtainTicketbooks, ())
            .await?;

        Ok(Response::new(AccountCommandResponse {
            error: result.err(),
        }))
    }

    async fn is_account_stored(&self, _request: Request<()>) -> Result<Response<bool>> {
        let is_stored = self
            .send_and_wait(VpnServiceCommand::IsAccountStored, ())
            .await?;

        Ok(Response::new(is_stored))
    }

    async fn get_account_mode(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Option<StoredAccountMode>>> {
        let mode = self
            .send_and_wait(VpnServiceCommand::GetAccountMode, ())
            .await?;

        Ok(Response::new(mode))
    }

    async fn forget_account(
        &self,
        _request: Request<()>,
    ) -> Result<Response<AccountCommandResponse>> {
        let result = self
            .send_and_wait(VpnServiceCommand::ForgetAccount, ())
            .await?;

        Ok(Response::new(AccountCommandResponse {
            error: result.err(),
        }))
    }

    async fn rotate_keys(&self, _request: Request<()>) -> Result<Response<AccountCommandResponse>> {
        let result = self
            .send_and_wait(VpnServiceCommand::RotateKeys, ())
            .await?;

        Ok(Response::new(AccountCommandResponse {
            error: result.err(),
        }))
    }

    async fn get_account_identity(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Option<String>>> {
        let account_identity = self
            .send_and_wait(VpnServiceCommand::GetAccountIdentity, ())
            .await?
            .map_err(|err| Status::internal(format!("Failed to get account identity: {err:?}")))?;

        Ok(Response::new(account_identity))
    }

    async fn get_canonical_account_identity(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Option<String>>> {
        let account_identity = self
            .send_and_wait(VpnServiceCommand::GetCanonicalAccountIdentity, ())
            .await?
            .map_err(|err| {
                Status::internal(format!("Failed to get canonical account identity: {err:?}"))
            })?;

        Ok(Response::new(account_identity))
    }

    async fn get_account_links(
        &self,
        request: Request<String>,
    ) -> Result<Response<ParsedAccountLinks>> {
        let account_links = self
            .send_and_wait(VpnServiceCommand::GetAccountLinks, request.into_inner())
            .await?
            .map_err(|err| Status::internal(format!("Failed to get account links: {err}")))?;

        Ok(Response::new(account_links))
    }

    async fn get_account_state(
        &self,
        _request: Request<()>,
    ) -> Result<Response<AccountControllerState>> {
        let account_controller_state = self
            .send_and_wait(VpnServiceCommand::GetAccountState, ())
            .await?;

        Ok(Response::new(account_controller_state))
    }

    async fn refresh_account_state(&self, request: Request<bool>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::RefreshAccountState, request.into_inner())
            .await?;

        Ok(Response::new(()))
    }

    async fn get_account_usage(&self, _request: Request<()>) -> Result<Response<Vec<NymVpnUsage>>> {
        let account_usage = self
            .send_and_wait(VpnServiceCommand::GetAccountUsage, ())
            .await?
            .map_err(|err| Status::internal(format!("Failed to get account usage: {err}")))?;

        Ok(Response::new(account_usage))
    }

    async fn reset_device_identity(
        &self,
        request: Request<Option<Vec<u8>>>,
    ) -> Result<Response<()>> {
        let seed: Option<[u8; 32]> = request
            .into_inner()
            .map(|seed| {
                seed.as_slice()
                    .try_into()
                    .map_err(|_| Status::invalid_argument("Seed must be 32 bytes long"))
            })
            .transpose()?;

        self.send_and_wait(VpnServiceCommand::ResetDeviceIdentity, seed)
            .await?
            .map_err(|err| Status::internal(format!("Failed to reset device identity: {err}")))?;

        Ok(Response::new(()))
    }

    async fn get_device_identity(&self, _request: Request<()>) -> Result<Response<Option<String>>> {
        let device_identity = self
            .send_and_wait(VpnServiceCommand::GetDeviceIdentity, ())
            .await?
            .map_err(|err| Status::internal(format!("Failed to get device identity: {err}")))?;

        Ok(Response::new(device_identity))
    }

    async fn get_devices(&self, _request: Request<()>) -> Result<Response<Vec<NymVpnDevice>>> {
        let devices = self
            .send_and_wait(VpnServiceCommand::GetDevices, ())
            .await?
            .map_err(|err| Status::internal(format!("Failed to get devices: {err}")))?;

        Ok(Response::new(devices))
    }

    async fn get_active_devices(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Vec<NymVpnDevice>>> {
        let devices = self
            .send_and_wait(VpnServiceCommand::GetActiveDevices, ())
            .await?
            .map_err(|err| Status::internal(format!("Failed to get active devices: {err}")))?;

        Ok(Response::new(devices))
    }

    async fn get_available_tickets(
        &self,
        _request: Request<()>,
    ) -> Result<Response<AvailableTickets>> {
        let available_ticketbooks = self
            .send_and_wait(VpnServiceCommand::GetAvailableTickets, ())
            .await?
            .map_err(|err| Status::internal(format!("Failed to get available tickets: {err}")))?;

        Ok(Response::new(AvailableTickets::from(available_ticketbooks)))
    }

    async fn restock_ticketbooks(&self, _request: Request<()>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::RestockTicketbooks, ())
            .await?
            .map_err(|err| Status::internal(format!("Failed to restock ticketbooks: {err}")))?;

        Ok(Response::new(()))
    }

    async fn get_account_summary(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Option<VpnAccountSummary>>> {
        let account_summary = self
            .send_and_wait(VpnServiceCommand::GetAccountSummary, ())
            .await?
            .map_err(|err| Status::internal(format!("Failed to get account summary: {err}")))?;

        Ok(Response::new(account_summary))
    }

    async fn handle_subscription_payment(&self, _request: Request<()>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::HandleSubscriptionPayment, ())
            .await?
            .map_err(|err| {
                Status::internal(format!("Failed to handle subscription payment: {err}"))
            })?;

        Ok(Response::new(()))
    }

    async fn get_deeplink(&self, request: Request<GetDeeplinkParams>) -> Result<Response<String>> {
        let url = self
            .send_and_wait(VpnServiceCommand::GetDeeplink, request.into_inner())
            .await?
            .map_err(|err| Status::internal(format!("Failed to get deeplink: {err}")))?;

        Ok(Response::new(url.to_string()))
    }

    async fn get_autologin_deeplink(
        &self,
        request: Request<GetDeeplinkParams>,
    ) -> Result<Response<AutologinResponse>> {
        let autologin = self
            .send_and_wait(
                VpnServiceCommand::GetAutologinDeeplink,
                request.into_inner(),
            )
            .await?
            .map_err(|err| Status::internal(format!("Failed to get autologin deeplink: {err}")))?;

        Ok(Response::new(autologin))
    }

    async fn deeplink_store_account(
        &self,
        request: Request<String>,
    ) -> Result<Response<AccountCommandResponse>> {
        let result = self
            .send_and_wait(
                VpnServiceCommand::DeeplinkStoreAccount,
                request.into_inner(),
            )
            .await?;

        Ok(Response::new(AccountCommandResponse {
            error: result.err(),
        }))
    }

    async fn get_log_path(&self, _request: Request<()>) -> Result<Response<Option<LogPath>>> {
        let log_path = self
            .send_and_wait(VpnServiceCommand::GetLogPath, ())
            .await?;

        Ok(Response::new(log_path))
    }

    async fn delete_log_file(&self, _request: Request<()>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::DeleteLogFile, ())
            .await?;

        Ok(Response::new(()))
    }

    async fn is_sentry_enabled(&self, _request: Request<()>) -> Result<Response<bool>> {
        let enabled = self
            .send_and_wait(VpnServiceCommand::IsSentryEnabled, ())
            .await?;

        Ok(Response::new(enabled))
    }

    async fn enable_sentry(&self, _request: Request<()>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::ToggleSentry, true)
            .await?
            .map_err(|err| {
                tracing::error!("Failed to enable sentry monitoring: {err}");
                Status::internal("failed to enable sentry")
            })?;

        Ok(Response::new(()))
    }

    async fn disable_sentry(&self, _request: Request<()>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::ToggleSentry, false)
            .await?
            .map_err(|err| {
                tracing::error!("Failed to disable sentry monitoring: {err}");
                Status::internal("failed to disable sentry")
            })?;

        Ok(Response::new(()))
    }

    async fn network_stats_set_enabled(&self, request: Request<bool>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::EnableNetStats, request.into_inner())
            .await
            .map_err(|e| {
                Status::internal(format!("Failed to enable/disable network statistics: {e}"))
            })?;

        Ok(Response::new(()))
    }

    async fn network_stats_allow_disconnected(
        &self,
        request: Request<bool>,
    ) -> Result<Response<()>> {
        self.send_and_wait(
            VpnServiceCommand::AllowDisconnectedNetStats,
            request.into_inner(),
        )
        .await
        .map_err(|e| {
            Status::internal(format!(
                "Failed to set network statistics allow_disconnected: {e}"
            ))
        })?;

        Ok(Response::new(()))
    }

    async fn network_stats_reset_seed(
        &self,
        request: Request<Option<String>>,
    ) -> Result<Response<()>> {
        let _ = self
            .send_and_wait(VpnServiceCommand::ResetNetStatsSeed, request.into_inner())
            .await
            .map_err(|e| {
                Status::internal(format!("Failed to reset network statistics seed: {e}"))
            })?;

        Ok(Response::new(()))
    }

    async fn network_stats_get_seed(
        &self,
        _request: Request<()>,
    ) -> Result<Response<NetworkStatisticsIdentity>> {
        let identity = self
            .send_and_wait(VpnServiceCommand::GetNetStatsSeed, ())
            .await?
            .map_err(|e| {
                Status::internal(format!("Failed to get network statistics identity: {e}"))
            })?;

        Ok(Response::new(identity))
    }

    async fn enable_socks5(&self, request: Request<EnableSocks5Request>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::EnableSocks5, request.into_inner())
            .await?
            .map_err(|err| {
                tracing::error!("Failed to enable SOCKS5 proxy: {err}");
                match err {
                    Socks5Error::GatewayNotSupported => Status::failed_precondition(
                        "Gateway does not support SOCKS5 network requester",
                    ),
                    Socks5Error::InvalidConfig(msg) => Status::failed_precondition(msg),
                    Socks5Error::LazySocks5Error(_) => {
                        Status::internal(format!("Failed to enable SOCKS5 proxy: {err}"))
                    }
                }
            })?;

        Ok(Response::new(()))
    }

    async fn disable_socks5(&self, _request: Request<()>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::DisableSocks5, ())
            .await?
            .map_err(|err| {
                tracing::error!("Failed to disable SOCKS5 proxy: {err}");
                Status::internal(format!("Failed to disable SOCKS5 proxy: {err}"))
            })?;

        Ok(Response::new(()))
    }

    async fn get_socks5_status(&self, _request: Request<()>) -> Result<Response<Socks5Status>> {
        let status = self
            .send_and_wait(VpnServiceCommand::GetSocks5Status, ())
            .await?
            .map_err(|err| {
                tracing::debug!("Failed to get SOCKS5 status: {err}");
                Status::internal(format!("Failed to get SOCKS5 status: {err}"))
            })?;

        Ok(Response::new(status))
    }

    async fn get_privy_derivation_message(
        &self,
        _request: Request<()>,
    ) -> Result<Response<PrivyDerivationMessage>> {
        Ok(Response::new(PrivyDerivationMessage {
            message: nym_vpn_lib::privy::message_to_sign(),
        }))
    }

    async fn run_diagnostic(
        &self,
        request: Request<DiagnosticRunParams>,
    ) -> Result<Response<DiagnosticReport>> {
        let report = self
            .send_and_wait(VpnServiceCommand::RunDiagnostic, request.into_inner())
            .await?;

        Ok(Response::new(report))
    }

    async fn register_diagnostic(
        &self,
        request: Request<DiagnosticRegisterParams>,
    ) -> Result<Response<RegistrationReport>> {
        let report = self
            .send_and_wait(VpnServiceCommand::RegisterDiagnostic, request.into_inner())
            .await?;

        Ok(Response::new(report))
    }

    async fn get_tentative_gateways(
        &self,
        _request: Request<()>,
    ) -> Result<Response<TentativeGateways>> {
        let tentative_gateways = self
            .send_and_wait(VpnServiceCommand::GetTentativeGateways, ())
            .await?;

        Ok(Response::new(tentative_gateways))
    }

    async fn is_split_tunnel_supported(&self, _request: Request<()>) -> Result<Response<bool>> {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            let is_available = self
                .send_and_wait(VpnServiceCommand::IsSplitTunnelSupported, ())
                .await?;
            Ok(Response::new(is_available))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        Err(Status::internal("Unsupported platform"))
    }

    async fn set_enable_split_tunnel(&self, _request: Request<bool>) -> Result<Response<()>> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            self.send_and_wait(
                VpnServiceCommand::SetEnableSplitTunnel,
                _request.into_inner(),
            )
            .await?;
            Ok(Response::new(()))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        Err(Status::internal("Unsupported platform"))
    }

    async fn add_split_tunnel_app(&self, _request: Request<SplitApp>) -> Result<Response<()>> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            self.send_and_wait(VpnServiceCommand::AddSplitTunnelApp, _request.into_inner())
                .await?;
            Ok(Response::new(()))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        Err(Status::internal("Unsupported platform"))
    }

    async fn remove_split_tunnel_app(&self, _request: Request<SplitApp>) -> Result<Response<()>> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            self.send_and_wait(
                VpnServiceCommand::RemoveSplitTunnelApp,
                _request.into_inner(),
            )
            .await?;
            Ok(Response::new(()))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        Err(Status::internal("Unsupported platform"))
    }

    async fn clear_split_tunnel_apps(&self, _request: Request<()>) -> Result<Response<()>> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            self.send_and_wait(VpnServiceCommand::ClearSplitTunnelApps, ())
                .await?;
            Ok(Response::new(()))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        Err(Status::internal("Unsupported platform"))
    }

    async fn get_split_tunnel_excluded_processes(
        &self,
        _request: Request<()>,
    ) -> Result<Response<SplitTunnelExcludedProcessList>> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let processes = self
                .send_and_wait(VpnServiceCommand::GetSplitTunnelExcludedProcesses, ())
                .await?;
            Ok(Response::new(processes))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        Err(Status::internal("Unsupported platform"))
    }

    async fn need_full_disk_permissions(&self, _request: Request<()>) -> Result<Response<bool>> {
        #[cfg(target_os = "macos")]
        {
            let need_fda = self
                .send_and_wait(VpnServiceCommand::NeedFullDiskPermissions, ())
                .await?;
            Ok(Response::new(need_fda))
        }

        #[cfg(not(target_os = "macos"))]
        Err(Status::internal("Unsupported platform"))
    }

    async fn add_split_tunnel_process(&self, _request: Request<i32>) -> Result<Response<()>> {
        #[cfg(target_os = "linux")]
        {
            self.send_and_wait(
                VpnServiceCommand::AddSplitTunnelProcess,
                _request.into_inner(),
            )
            .await?;
            Ok(Response::new(()))
        }

        #[cfg(not(target_os = "linux"))]
        Err(Status::internal("Unsupported platform"))
    }

    async fn remove_split_tunnel_process(&self, _request: Request<i32>) -> Result<Response<()>> {
        #[cfg(target_os = "linux")]
        {
            self.send_and_wait(
                VpnServiceCommand::RemoveSplitTunnelProcess,
                _request.into_inner(),
            )
            .await?;
            Ok(Response::new(()))
        }

        #[cfg(not(target_os = "linux"))]
        Err(Status::internal("Unsupported platform"))
    }

    async fn clear_split_tunnel_processes(&self, _request: Request<()>) -> Result<Response<()>> {
        #[cfg(target_os = "linux")]
        {
            self.send_and_wait(VpnServiceCommand::ClearSplitTunnelProcesses, ())
                .await?;
            Ok(Response::new(()))
        }

        #[cfg(not(target_os = "linux"))]
        Err(Status::internal("Unsupported platform"))
    }

    async fn set_profile(&self, request: Request<ProfileOptions>) -> Result<Response<()>> {
        self.send_and_wait(VpnServiceCommand::SetProfile, request.into_inner().profile)
            .await
            .map_err(|e| Status::internal(format!("Failed to set profile: {e}")))?;

        Ok(Response::new(()))
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

#[cfg(test)]
mod tests {
    use nym_vpn_lib_types::{HttpRpcSettings, Socks5Settings, Socks5State};
    use nym_vpn_rpc::NymVpnServiceClient;
    use tokio_stream::wrappers::TcpListenerStream;

    use super::*;

    fn enable_request() -> EnableSocks5Request {
        EnableSocks5Request {
            socks5_settings: Socks5Settings {
                listen_address: Some("127.0.0.1:1080".parse().unwrap()),
            },
            http_rpc_settings: HttpRpcSettings {
                listen_address: None,
            },
            exit_point: ExitPoint::Country {
                two_letter_iso_country_code: "CH".to_owned(),
            },
        }
    }

    async fn handle_commands(mut command_rx: UnboundedReceiver<VpnServiceCommand>) {
        let mut socks5_status = None;
        while let Some(command) = command_rx.recv().await {
            match command {
                VpnServiceCommand::EnableSocks5(tx, request) => {
                    socks5_status = Some(Socks5Status {
                        state: Socks5State::Idle,
                        socks5_settings: request.socks5_settings,
                        http_rpc_settings: request.http_rpc_settings,
                        error_message: None,
                        active_connections: 0,
                    });
                    let _ = tx.send(Ok(()));
                }
                VpnServiceCommand::GetSocks5Status(tx, ()) => {
                    let _ = tx.send(
                        socks5_status
                            .clone()
                            .ok_or(Socks5Error::InvalidConfig("disabled".to_owned())),
                    );
                }
                VpnServiceCommand::GetTunnelState(tx, ()) => {
                    let _ = tx.send(TunnelState::Disconnected);
                }
                VpnServiceCommand::SetAllowLan(tx, _) => {
                    let _ = tx.send(());
                }
                _ => panic!("unexpected command"),
            }
        }
    }

    async fn start_server() -> (
        NymVpnServiceClient<tonic::transport::Channel>,
        broadcast::Sender<TunnelEvent>,
    ) {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = broadcast::channel(4);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(handle_commands(command_rx));
        tokio::spawn(
            Server::builder()
                .add_service(NymVpnServiceServer::new(CommandInterface::new(
                    command_tx, event_rx,
                )))
                .serve_with_incoming(TcpListenerStream::new(listener)),
        );

        let client = NymVpnServiceClient::connect(format!("http://{addr}"))
            .await
            .unwrap();
        (client, event_tx)
    }

    #[tokio::test]
    async fn unary_calls_round_trip_domain_types() {
        let (mut client, _event_tx) = start_server().await;

        client.set_allow_lan(true).await.unwrap();
        assert_eq!(
            client.get_tunnel_state(()).await.unwrap().into_inner(),
            TunnelState::Disconnected
        );

        let request = enable_request();
        client.enable_socks5(request.clone()).await.unwrap();
        let status = client.get_socks5_status(()).await.unwrap().into_inner();
        assert_eq!(status.state, Socks5State::Idle);
        assert_eq!(status.socks5_settings, request.socks5_settings);
    }

    #[tokio::test]
    async fn daemon_errors_map_to_status_codes() {
        let (mut client, _event_tx) = start_server().await;

        let error = client.get_socks5_status(()).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Internal);

        let error = client.set_geo_exclusion_listen_port(0).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn tunnel_events_are_streamed() {
        let (mut client, event_tx) = start_server().await;

        let mut events = client.listen_to_events(()).await.unwrap().into_inner();
        event_tx
            .send(TunnelEvent::NewState(TunnelState::Disconnected))
            .unwrap();

        let event = events.next().await.unwrap().unwrap();
        assert!(matches!(
            event,
            TunnelEvent::NewState(TunnelState::Disconnected)
        ));
    }
}
