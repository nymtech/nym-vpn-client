import Foundation
import TunnelStatus

public enum StatusInfoState {
    case initialising
    case connecting
    case connectionTime
    case error(message: String)
    case unknown

    public init(tunnelStatus: TunnelStatus) {
        switch tunnelStatus {
        case .connected:
            self = .connectionTime
        case .connecting, .reasserting, .restarting:
            self = .connecting
        case .disconnected, .disconnecting:
            self = .unknown
        }
    }

    var localizedTitle: String {
        switch self {
        case .initialising:
            "initializingClient".localizedString
        case .connecting:
            "establishingConnection".localizedString
        case .connectionTime:
            "connectionTime".localizedString
        case let .error(message):
            message
        case .unknown:
            // Empty string hides the view. To not mess up UX spacing - need 'space' to still show it.
            " "
        }
    }
}
