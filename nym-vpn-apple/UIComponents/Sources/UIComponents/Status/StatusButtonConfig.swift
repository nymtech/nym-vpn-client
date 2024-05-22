import SwiftUI
import Theme
import TunnelStatus

public enum StatusButtonConfig: String {
    case connected
    case connecting
    case disconnecting
    case disconnected
    case error

    public init(tunnelStatus: TunnelStatus) {
        switch tunnelStatus {
        case .connected:
            self = .connected
        case .connecting, .reasserting, .restarting:
            self = .connecting
        case .disconnected:
            self = .disconnected
        case .disconnecting:
            self = .disconnecting
        }
    }

    var title: String {
        self.rawValue.localizedString
    }

    var textColor: Color {
        switch self {
        case .connected:
            return NymColor.confirm
        case .connecting, .disconnecting:
            return NymColor.sysOnSurfaceWhite
        case .disconnected, .error:
            return NymColor.sysOnSecondary
        }
    }

    var backgroundColor: Color {
        switch self {
        case .connected:
            return NymColor.statusGreen
        case .connecting, .disconnecting, .disconnected, .error:
            return NymColor.statusButtonBackground
        }
    }
}
