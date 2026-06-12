#if os(iOS)
import Foundation
import NymVPNLib
import Theme

public enum ProcessingAccountErrorMapper {
    private static let somethingWentWrong = "generalNymError.somethingWentWrong".localizedString

    /// Safe one-line description for app log files (no secrets).
    public static func logSafeDescription(for error: Error) -> String {
        if let vpnError = error as? VpnError {
            return "VpnError.\(vpnErrorLogLabel(vpnError))"
        }
        if let localized = error as? LocalizedError,
           let description = localized.errorDescription {
            return String(description.prefix(200))
        }
        return "Error(\(String(describing: Swift.type(of: error)))"
    }

    public static func localizedMessage(for error: Error) -> String {
        if let vpnError = error as? VpnError {
            return localizedMessage(for: vpnError)
        }
        if let localized = error as? LocalizedError,
           let description = localized.errorDescription,
           !description.isEmpty {
            return description
        }
        return somethingWentWrong
    }

    public static func localizedMessage(for vpnError: VpnError) -> String {
        switch vpnError {
        case let .InternalError(details: details):
            if details.contains("Maximum number of devices reached") {
                return "errorReason.maxDevicesReached".localizedString
            }
            if details.contains("Fair usage depleted") {
                return "errorReason.bandwidthExceeded".localizedString
            }
            if details.contains("Device time is desynced") {
                return "errorReason.deviceTimeOutOfSync".localizedString
            }
            return details
        case let .ZkNymAcquisitionFailure(details: details):
            if details.contains("BandwidthExceeded") || details.contains("Fair usage depleted") {
                return "errorReason.bandwidthExceeded".localizedString
            }
            if details.contains("device_not_authenticated") {
                return "errorReason.noDeviceStored".localizedString
            }
            if details.contains("Maximum number of devices reached") {
                return "errorReason.maxDevicesReached".localizedString
            }
            return VPNErrorReason(with: vpnError).description ?? somethingWentWrong
        default:
            return VPNErrorReason(with: vpnError).description ?? somethingWentWrong
        }
    }

    private static func vpnErrorLogLabel(_ vpnError: VpnError) -> String {
        switch vpnError {
        case let .InternalError(details: details):
            return "InternalError(\(details.prefix(120)))"
        case let .ZkNymAcquisitionFailure(details: details):
            return "ZkNymAcquisitionFailure(\(details.prefix(120)))"
        case let .Storage(details: details):
            return "Storage(\(details.prefix(120)))"
        case let .FailedAccountRegistration(details: details):
            return "FailedAccountRegistration(\(details.prefix(120)))"
        case .AccountStoreBusy:
            return "AccountStoreBusy"
        default:
            return String(describing: vpnError)
        }
    }
}
#endif
