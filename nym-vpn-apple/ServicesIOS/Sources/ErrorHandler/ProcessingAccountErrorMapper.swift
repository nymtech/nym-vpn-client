#if os(iOS)
import Foundation
import NymVPNLib
import Theme

public enum ProcessingAccountErrorMapper {
    private static let somethingWentWrong = "generalNymError.somethingWentWrong".localizedString

    public static func localizedMessage(for error: Error) -> String {
        guard let vpnError = error as? VpnError else {
            return somethingWentWrong
        }
        return localizedMessage(for: vpnError)
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
}
#endif
