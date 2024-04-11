import SwiftUI
import Theme
import TunnelStatus

public enum ConnectButtonState {
    case connect
    case disconnect
    case disconnecting
    case stop

    public init(tunnelStatus: TunnelStatus) {
        switch tunnelStatus {
        case .connected:
            self = .disconnect
        case .connecting, .reasserting, .restarting:
            self = .stop
        case .disconnected:
            self = .connect
        case .disconnecting:
            self = .disconnecting
        }
    }

    public var localizedTitle: String {
        switch self {
        case .connect:
            "connect".localizedString
        case .disconnect:
            "disconnect".localizedString
        case .disconnecting:
            "disconnecting".localizedString
        case .stop:
            "stop".localizedString
        }
    }

    var backgroundColor: Color {
        switch self {
        case .connect:
            NymColor.primaryOrange
        case .disconnect:
            NymColor.disconnect
        case .stop, .disconnecting:
            NymColor.sysSecondary
        }
    }
}
