// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use nym_vpn_lib_types::{
    AccountBalanceResponse, AccountCommandResponse, AccountControllerState, AppCompatibility,
    AutologinResponse, AvailableTickets, BridgeMode, DiagnosticRegisterParams, DiagnosticReport,
    DiagnosticRunParams, EnableSocks5Request, EntryPoint, ExitPoint, FeatureFlags, FrontingMode,
    Gateway, GetDeeplinkParams, GetRecentGatewaysParams, ListGatewaysOptions, LogPath,
    LookupGatewayFilters, MixnetTrafficConfig, NetworkStatisticsIdentity, NymVpnDevice,
    NymVpnUsage, ParsedAccountLinks, PrivyDerivationMessage, ProfileOptions, RecentGateways,
    RegistrationReport, Socks5Status, SplitApp, SplitTunnelExcludedProcessList,
    StoreAccountRequest, StoredAccountMode, SystemMessage, TentativeGateways, TunnelEvent,
    TunnelState, VpnAccountSummary, VpnServiceConfig, VpnServiceInfo,
};

use crate::{Empty, macros::service};

service! {
    package nym_vpn;
    service NymVpnService;

    rpc Info(Empty) -> VpnServiceInfo;
    rpc GetConfig(Empty) -> VpnServiceConfig;
    rpc SetEntryPoint(EntryPoint) -> Empty;
    rpc SetExitPoint(ExitPoint) -> Empty;
    rpc SetDisableIpv6(bool) -> Empty;
    rpc SetEnableTwoHop(bool) -> Empty;
    rpc SetBridgeMode(BridgeMode) -> Empty;
    rpc SetEnableAdBlocking(bool) -> Empty;
    rpc SetEnableConflictDetection(bool) -> Empty;
    rpc SetNetstack(bool) -> Empty;
    rpc SetAllowLan(bool) -> Empty;
    rpc SetResidentialExit(bool) -> Empty;
    rpc SetEnableCustomDns(bool) -> Empty;
    rpc SetCustomDns(Vec<IpAddr>) -> Empty;
    rpc SetMixnetTrafficConfig(MixnetTrafficConfig) -> Empty;
    rpc SetGeoExclusionEnabled(bool) -> Empty;
    rpc SetGeoExclusionListenPort(u16) -> Empty;
    rpc SetGeoExclusionExcludedCountries(Vec<String>) -> Empty;
    rpc SetEnableGeoLocation(bool) -> Empty;
    rpc SetEnableGatewayIndependence(bool) -> Empty;
    rpc SetGatewayIndependenceNotifications(bool) -> Empty;
    rpc SetFrontingMode(FrontingMode) -> Empty;
    rpc SetNetwork(String) -> Empty;
    rpc GetSystemMessages(Empty) -> Vec<SystemMessage>;
    rpc GetAppCompatibility(Empty) -> Option<AppCompatibility>;
    rpc GetFeatureFlags(Empty) -> FeatureFlags;
    rpc GetDefaultDns(Empty) -> Vec<IpAddr>;
    rpc ConnectTunnel(Empty) -> bool;
    rpc ReconnectTunnel(Empty) -> bool;
    rpc DisconnectTunnel(Empty) -> bool;
    rpc GetTunnelState(Empty) -> TunnelState;
    stream ListenToEvents(Empty) -> TunnelEvent;
    rpc ListGateways(ListGatewaysOptions) -> Vec<Gateway>;
    rpc ListFilteredGateways(LookupGatewayFilters) -> Vec<Gateway>;
    rpc StoreAccount(StoreAccountRequest) -> AccountCommandResponse;
    rpc AccountBalance(Empty) -> AccountBalanceResponse;
    rpc DecentralisedObtainTicketbooks(Empty) -> AccountCommandResponse;
    rpc IsAccountStored(Empty) -> bool;
    rpc ForgetAccount(Empty) -> AccountCommandResponse;
    rpc RotateKeys(Empty) -> AccountCommandResponse;
    rpc GetAccountIdentity(Empty) -> Option<String>;
    rpc GetCanonicalAccountIdentity(Empty) -> Option<String>;
    rpc GetAccountMode(Empty) -> Option<StoredAccountMode>;
    rpc GetAccountLinks(String) -> ParsedAccountLinks;
    rpc GetAccountState(Empty) -> AccountControllerState;
    rpc RefreshAccountState(bool) -> Empty;
    rpc GetAccountUsage(Empty) -> Vec<NymVpnUsage>;
    rpc ResetDeviceIdentity(Option<Vec<u8>>) -> Empty;
    rpc GetDeviceIdentity(Empty) -> Option<String>;
    rpc GetDevices(Empty) -> Vec<NymVpnDevice>;
    rpc GetActiveDevices(Empty) -> Vec<NymVpnDevice>;
    rpc GetAvailableTickets(Empty) -> AvailableTickets;
    rpc RestockTicketbooks(Empty) -> Empty;
    rpc GetAccountSummary(Empty) -> Option<VpnAccountSummary>;
    rpc HandleSubscriptionPayment(Empty) -> Empty;
    rpc GetDeeplink(GetDeeplinkParams) -> String;
    rpc GetAutologinDeeplink(GetDeeplinkParams) -> AutologinResponse;
    rpc DeeplinkStoreAccount(String) -> AccountCommandResponse;
    rpc GetLogPath(Empty) -> Option<LogPath>;
    rpc DeleteLogFile(Empty) -> Empty;
    rpc IsSentryEnabled(Empty) -> bool;
    rpc EnableSentry(Empty) -> Empty;
    rpc DisableSentry(Empty) -> Empty;
    rpc NetworkStatsSetEnabled(bool) -> Empty;
    rpc NetworkStatsAllowDisconnected(bool) -> Empty;
    rpc NetworkStatsResetSeed(Option<String>) -> Empty;
    rpc NetworkStatsGetSeed(Empty) -> NetworkStatisticsIdentity;
    rpc EnableSocks5(EnableSocks5Request) -> Empty;
    rpc DisableSocks5(Empty) -> Empty;
    rpc GetSocks5Status(Empty) -> Socks5Status;
    rpc GetPrivyDerivationMessage(Empty) -> PrivyDerivationMessage;
    rpc RunDiagnostic(DiagnosticRunParams) -> DiagnosticReport;
    rpc RegisterDiagnostic(DiagnosticRegisterParams) -> RegistrationReport;
    rpc GetTentativeGateways(Empty) -> TentativeGateways;
    rpc GetRecentGateways(GetRecentGatewaysParams) -> RecentGateways;
    rpc IsSplitTunnelSupported(Empty) -> bool;
    rpc SetEnableSplitTunnel(bool) -> Empty;
    rpc AddSplitTunnelApp(SplitApp) -> Empty;
    rpc RemoveSplitTunnelApp(SplitApp) -> Empty;
    rpc ClearSplitTunnelApps(Empty) -> Empty;
    rpc GetSplitTunnelExcludedProcesses(Empty) -> SplitTunnelExcludedProcessList;
    rpc AddSplitTunnelProcess(i32) -> Empty;
    rpc RemoveSplitTunnelProcess(i32) -> Empty;
    rpc ClearSplitTunnelProcesses(Empty) -> Empty;
    rpc NeedFullDiskPermissions(Empty) -> bool;
    rpc SetProfile(ProfileOptions) -> Empty;
}
