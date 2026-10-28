// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tonic_build::manual::{Builder, Method, Service};

const EMPTY: &str = "crate::Empty";
const BOOL: &str = "bool";
const I32: &str = "i32";
const U16: &str = "u16";
const STRING: &str = "::std::string::String";
const OPTION_STRING: &str = "::core::option::Option<::std::string::String>";
const STRING_LIST: &str = "::std::vec::Vec<::std::string::String>";
const IP_ADDR_LIST: &str = "::std::vec::Vec<::std::net::IpAddr>";
const GATEWAY_LIST: &str = "::std::vec::Vec<::nym_vpn_lib_types::Gateway>";
const ACCOUNT_COMMAND_RESPONSE: &str = "::nym_vpn_lib_types::AccountCommandResponse";
const DEVICE_LIST: &str = "::std::vec::Vec<::nym_vpn_lib_types::NymVpnDevice>";

enum Kind {
    Unary,
    ServerStreaming,
}

struct Rpc {
    route_name: &'static str,
    input_type: &'static str,
    output_type: &'static str,
    kind: Kind,
}

const fn unary(
    route_name: &'static str,
    input_type: &'static str,
    output_type: &'static str,
) -> Rpc {
    Rpc {
        route_name,
        input_type,
        output_type,
        kind: Kind::Unary,
    }
}

const fn server_streaming(
    route_name: &'static str,
    input_type: &'static str,
    output_type: &'static str,
) -> Rpc {
    Rpc {
        route_name,
        input_type,
        output_type,
        kind: Kind::ServerStreaming,
    }
}

const RPCS: &[Rpc] = &[
    unary("Info", EMPTY, "::nym_vpn_lib_types::VpnServiceInfo"),
    unary("GetConfig", EMPTY, "::nym_vpn_lib_types::VpnServiceConfig"),
    unary("SetEntryPoint", "::nym_vpn_lib_types::EntryPoint", EMPTY),
    unary("SetExitPoint", "::nym_vpn_lib_types::ExitPoint", EMPTY),
    unary("SetDisableIpv6", BOOL, EMPTY),
    unary("SetEnableTwoHop", BOOL, EMPTY),
    unary("SetBridgeMode", "::nym_vpn_lib_types::BridgeMode", EMPTY),
    unary("SetEnableAdBlocking", BOOL, EMPTY),
    unary("SetEnableConflictDetection", BOOL, EMPTY),
    unary("SetNetstack", BOOL, EMPTY),
    unary("SetAllowLan", BOOL, EMPTY),
    unary("SetResidentialExit", BOOL, EMPTY),
    unary("SetEnableCustomDns", BOOL, EMPTY),
    unary("SetCustomDns", IP_ADDR_LIST, EMPTY),
    unary(
        "SetMixnetTrafficConfig",
        "::nym_vpn_lib_types::MixnetTrafficConfig",
        EMPTY,
    ),
    unary("SetGeoExclusionEnabled", BOOL, EMPTY),
    unary("SetGeoExclusionListenPort", U16, EMPTY),
    unary("SetGeoExclusionExcludedCountries", STRING_LIST, EMPTY),
    unary("SetEnableGeoLocation", BOOL, EMPTY),
    unary("SetEnableGatewayIndependence", BOOL, EMPTY),
    unary("SetGatewayIndependenceNotifications", BOOL, EMPTY),
    unary(
        "SetFrontingMode",
        "::nym_vpn_lib_types::FrontingMode",
        EMPTY,
    ),
    unary("SetNetwork", STRING, EMPTY),
    unary(
        "GetSystemMessages",
        EMPTY,
        "::std::vec::Vec<::nym_vpn_lib_types::SystemMessage>",
    ),
    unary(
        "GetAppCompatibility",
        EMPTY,
        "::core::option::Option<::nym_vpn_lib_types::AppCompatibility>",
    ),
    unary(
        "GetFeatureFlags",
        EMPTY,
        "::nym_vpn_lib_types::FeatureFlags",
    ),
    unary("GetDefaultDns", EMPTY, IP_ADDR_LIST),
    unary("ConnectTunnel", EMPTY, BOOL),
    unary("ReconnectTunnel", EMPTY, BOOL),
    unary("DisconnectTunnel", EMPTY, BOOL),
    unary("GetTunnelState", EMPTY, "::nym_vpn_lib_types::TunnelState"),
    server_streaming("ListenToEvents", EMPTY, "::nym_vpn_lib_types::TunnelEvent"),
    unary(
        "ListGateways",
        "::nym_vpn_lib_types::ListGatewaysOptions",
        GATEWAY_LIST,
    ),
    unary(
        "ListFilteredGateways",
        "::nym_vpn_lib_types::LookupGatewayFilters",
        GATEWAY_LIST,
    ),
    unary(
        "StoreAccount",
        "::nym_vpn_lib_types::StoreAccountRequest",
        ACCOUNT_COMMAND_RESPONSE,
    ),
    unary(
        "AccountBalance",
        EMPTY,
        "::nym_vpn_lib_types::AccountBalanceResponse",
    ),
    unary(
        "DecentralisedObtainTicketbooks",
        EMPTY,
        ACCOUNT_COMMAND_RESPONSE,
    ),
    unary("IsAccountStored", EMPTY, BOOL),
    unary("ForgetAccount", EMPTY, ACCOUNT_COMMAND_RESPONSE),
    unary("RotateKeys", EMPTY, ACCOUNT_COMMAND_RESPONSE),
    unary("GetAccountIdentity", EMPTY, OPTION_STRING),
    unary("GetCanonicalAccountIdentity", EMPTY, OPTION_STRING),
    unary(
        "GetAccountMode",
        EMPTY,
        "::core::option::Option<::nym_vpn_lib_types::StoredAccountMode>",
    ),
    unary(
        "GetAccountLinks",
        STRING,
        "::nym_vpn_lib_types::ParsedAccountLinks",
    ),
    unary(
        "GetAccountState",
        EMPTY,
        "::nym_vpn_lib_types::AccountControllerState",
    ),
    unary("RefreshAccountState", BOOL, EMPTY),
    unary(
        "GetAccountUsage",
        EMPTY,
        "::std::vec::Vec<::nym_vpn_lib_types::NymVpnUsage>",
    ),
    unary(
        "ResetDeviceIdentity",
        "::core::option::Option<::std::vec::Vec<u8>>",
        EMPTY,
    ),
    unary("GetDeviceIdentity", EMPTY, OPTION_STRING),
    unary("GetDevices", EMPTY, DEVICE_LIST),
    unary("GetActiveDevices", EMPTY, DEVICE_LIST),
    unary(
        "GetAvailableTickets",
        EMPTY,
        "::nym_vpn_lib_types::AvailableTickets",
    ),
    unary("RestockTicketbooks", EMPTY, EMPTY),
    unary(
        "GetAccountSummary",
        EMPTY,
        "::core::option::Option<::nym_vpn_lib_types::VpnAccountSummary>",
    ),
    unary("HandleSubscriptionPayment", EMPTY, EMPTY),
    unary(
        "GetDeeplink",
        "::nym_vpn_lib_types::GetDeeplinkParams",
        STRING,
    ),
    unary(
        "GetAutologinDeeplink",
        "::nym_vpn_lib_types::GetDeeplinkParams",
        "::nym_vpn_lib_types::AutologinResponse",
    ),
    unary("DeeplinkStoreAccount", STRING, ACCOUNT_COMMAND_RESPONSE),
    unary(
        "GetLogPath",
        EMPTY,
        "::core::option::Option<::nym_vpn_lib_types::LogPath>",
    ),
    unary("DeleteLogFile", EMPTY, EMPTY),
    unary("IsSentryEnabled", EMPTY, BOOL),
    unary("EnableSentry", EMPTY, EMPTY),
    unary("DisableSentry", EMPTY, EMPTY),
    unary("NetworkStatsSetEnabled", BOOL, EMPTY),
    unary("NetworkStatsAllowDisconnected", BOOL, EMPTY),
    unary("NetworkStatsResetSeed", OPTION_STRING, EMPTY),
    unary(
        "NetworkStatsGetSeed",
        EMPTY,
        "::nym_vpn_lib_types::NetworkStatisticsIdentity",
    ),
    unary(
        "EnableSocks5",
        "::nym_vpn_lib_types::EnableSocks5Request",
        EMPTY,
    ),
    unary("DisableSocks5", EMPTY, EMPTY),
    unary(
        "GetSocks5Status",
        EMPTY,
        "::nym_vpn_lib_types::Socks5Status",
    ),
    unary(
        "GetPrivyDerivationMessage",
        EMPTY,
        "::nym_vpn_lib_types::PrivyDerivationMessage",
    ),
    unary(
        "RunDiagnostic",
        "::nym_vpn_lib_types::DiagnosticRunParams",
        "::nym_vpn_lib_types::DiagnosticReport",
    ),
    unary(
        "RegisterDiagnostic",
        "::nym_vpn_lib_types::DiagnosticRegisterParams",
        "::nym_vpn_lib_types::RegistrationReport",
    ),
    unary(
        "GetTentativeGateways",
        EMPTY,
        "::nym_vpn_lib_types::TentativeGateways",
    ),
    unary(
        "GetRecentGateways",
        "::nym_vpn_lib_types::GetRecentGatewaysParams",
        "::nym_vpn_lib_types::RecentGateways",
    ),
    unary("IsSplitTunnelSupported", EMPTY, BOOL),
    unary("SetEnableSplitTunnel", BOOL, EMPTY),
    unary("AddSplitTunnelApp", "::nym_vpn_lib_types::SplitApp", EMPTY),
    unary(
        "RemoveSplitTunnelApp",
        "::nym_vpn_lib_types::SplitApp",
        EMPTY,
    ),
    unary("ClearSplitTunnelApps", EMPTY, EMPTY),
    unary(
        "GetSplitTunnelExcludedProcesses",
        EMPTY,
        "::nym_vpn_lib_types::SplitTunnelExcludedProcessList",
    ),
    unary("AddSplitTunnelProcess", I32, EMPTY),
    unary("RemoveSplitTunnelProcess", I32, EMPTY),
    unary("ClearSplitTunnelProcesses", EMPTY, EMPTY),
    unary("NeedFullDiskPermissions", EMPTY, BOOL),
    unary("SetProfile", "::nym_vpn_lib_types::ProfileOptions", EMPTY),
];

fn snake_case(route_name: &str) -> String {
    let mut name = String::new();
    for (i, ch) in route_name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i != 0 {
                name.push('_');
            }
            name.push(ch.to_ascii_lowercase());
        } else {
            name.push(ch);
        }
    }
    name
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let service = RPCS
        .iter()
        .fold(
            Service::builder().name("NymVpnService").package("nym_vpn"),
            |service, rpc| {
                let method = Method::builder()
                    .name(snake_case(rpc.route_name))
                    .route_name(rpc.route_name)
                    .input_type(rpc.input_type)
                    .output_type(rpc.output_type)
                    .codec_path("crate::codec::CborCodec");
                let method = match rpc.kind {
                    Kind::Unary => method,
                    Kind::ServerStreaming => method.server_streaming(),
                };
                service.method(method.build())
            },
        )
        .build();

    Builder::new().compile(&[service]);
}
