import Constants

extension GRPCManager {
    // swiftlint:disable:next function_body_length
    func convertToGeneralNymError(from error: Nym_Vpn_Error) -> GeneralNymError {
        switch error.kind {
        case .unspecified, .unhandled:
            GeneralNymError.library(message: "\("error.unexpected".localizedString): \(error.message)")
        case .noValidCredentials:
            GeneralNymError.library(message: error.message)
        case .timeout:
            GeneralNymError.library(message: "error.timeout".localizedString)
        case .gatewayDirectory:
            GeneralNymError.library(message: "error.gatewayDirectory".localizedString)
        case .UNRECOGNIZED(let code):
            GeneralNymError.library(message: "error.unrecognized".localizedString + " \(code)")
        case .mixnetTimeout:
            // TODO: localize errors
            GeneralNymError.library(message: error.message)
        case .gatewayDirectoryLookupGateways:
            GeneralNymError.library(message: error.message)
        case .gatewayDirectoryLookupGatewayIdentity:
            GeneralNymError.library(message: error.message)
        case .gatewayDirectoryLookupRouterAddress:
            GeneralNymError.library(message: error.message)
        case .gatewayDirectoryLookupIp:
            GeneralNymError.library(message: error.message)
        case .gatewayDirectoryEntry:
            GeneralNymError.library(message: error.message)
        case .gatewayDirectoryEntryLocation:
            GeneralNymError.library(message: error.message)
        case .gatewayDirectoryExit:
            GeneralNymError.library(message: error.message)
        case .gatewayDirectoryExitLocation:
            GeneralNymError.library(message: error.message)
        case .gatewayDirectorySameEntryAndExitGw:
            GeneralNymError.library(message: "errorReason.sameEntryAndExitGateway".localizedString)
        case .outOfBandwidth:
            GeneralNymError.library(message: error.message)
        case .mixnetStoragePaths:
            GeneralNymError.library(message: error.message)
        case .mixnetDefaultStorage:
            GeneralNymError.library(message: error.message)
        case .mixnetBuildClient:
            GeneralNymError.library(message: error.message)
        case .mixnetConnect:
            GeneralNymError.library(message: error.message)
        case .mixnetEntryGateway:
            GeneralNymError.library(message: error.message)
        case .gatewayDirectoryEntryID:
            GeneralNymError.library(message: error.message)
        case .iprFailedToConnect:
            GeneralNymError.library(message: error.message)
        case .outOfBandwidthWhenSettingUpTunnel:
            GeneralNymError.library(message: error.message)
        case .bringInterfaceUp:
            GeneralNymError.library(message: error.message)
        case .firewallInit:
            GeneralNymError.library(message: error.message)
        case .firewallResetPolicy:
            GeneralNymError.library(message: error.message)
        case .dnsInit:
            GeneralNymError.library(message: error.message)
        case .dnsSet:
            GeneralNymError.library(message: error.message)
        case .findDefaultInterface:
            GeneralNymError.library(message: error.message)
        case .unhandledExit:
            GeneralNymError.library(message: error.message)
        case .internal:
            GeneralNymError.library(message: error.message)
        case .authenticatorFailedToConnect:
            GeneralNymError.library(message: error.message)
        case .authenticatorConnectTimeout:
            GeneralNymError.library(message: error.message)
        case .authenticatorInvalidResponse:
            GeneralNymError.library(message: error.message)
        case .authenticatorRegistrationDataVerification:
            GeneralNymError.library(message: error.message)
        case .authenticatorEntryGatewaySocketAddr:
            GeneralNymError.library(message: error.message)
        case .authenticatorEntryGatewayIpv4:
            GeneralNymError.library(message: error.message)
        case .authenticatorWrongVersion:
            GeneralNymError.library(message: error.message)
        case .authenticatorMalformedReply:
            GeneralNymError.library(message: error.message)
        case .authenticatorAddressNotFound:
            GeneralNymError.library(message: error.message)
        case .authenticatorAuthenticationNotPossible:
            GeneralNymError.library(message: error.message)
        case .addIpv6Route:
            GeneralNymError.library(message: error.message)
        case .tun:
            GeneralNymError.library(message: error.message)
        case .routing:
            GeneralNymError.library(message: error.message)
        case .wireguardConfig:
            GeneralNymError.library(message: error.message)
        case .mixnetConnectionMonitor:
            GeneralNymError.library(message: error.message)
        }
    }
}
