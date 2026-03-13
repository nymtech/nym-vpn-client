import Foundation
import NetworkExtension

public enum TunnelsManagerError: LocalizedError {
    case emptyName
    case alreadyExists
    case tunnelList(error: Error)
    case addTunnel(error: Error)
    case modifyTunnel(error: Error)
    case removeTunnel(error: Error)

    public var errorDescription: String? {
        switch self {
        case .addTunnel(let error) where Self.isVPNPermissionDenied(error):
            String(
                localized: "error.vpnPermissionDenied",
                defaultValue: "VPN permission was not granted. Please allow the VPN configuration to connect.",
                bundle: .main
            )
        case .addTunnel(let error), .tunnelList(let error), .modifyTunnel(let error), .removeTunnel(let error):
            error.localizedDescription
        case .emptyName, .alreadyExists:
            String(
                localized: "error.unexpected",
                defaultValue: "An unexpected error occurred.",
                bundle: .main
            )
        }
    }

    private static func isVPNPermissionDenied(_ error: Error) -> Bool {
        let nsError = error as NSError
        // User denied/cancelled VPN profile installation
        return nsError.domain == NEVPNErrorDomain
            && nsError.code == NEVPNError.configurationReadWriteFailed.rawValue
    }
}
