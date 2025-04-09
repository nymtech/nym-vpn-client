import SwiftUI
import Theme
import TunnelStatus

public enum StatusButtonConfig: String {
    case connected
    case connecting
    case disconnecting
    case disconnected
    case noInternet
    case noInternetReconnect
    case error

    public init(tunnelStatus: TunnelStatus, hasInternet: Bool) {
        if !hasInternet {
            self = .noInternet
            return
        }
        switch tunnelStatus {
        case .connected:
            self = .connected
        case .connecting, .reasserting, .restarting:
            self = .connecting
        case .disconnected:
            self = .disconnected
        case .disconnecting:
            self = .disconnecting
        case .offline, .unknown:
            self = .noInternet
        case .offlineReconnect:
            self = .noInternetReconnect
        case .error:
            self = .error
        }
    }

    var title: String {
        self.rawValue.localizedString
    }

    var textColor: Color {
        switch self {
        case .connected:
            NymColor.action
        case .connecting, .disconnecting, .noInternet, .noInternetReconnect:
            NymColor.primary
        case .disconnected:
            NymColor.gray1
        case .error:
            NymColor.black
        }
    }

    var backgroundColor: Color {
        switch self {
        case .connected:
            NymColor.action.opacity(0.1)
        case .connecting, .disconnecting, .disconnected:
            NymColor.backgroundHover
        case .noInternet, .noInternetReconnect, .error:
            NymColor.error
        }
    }
}
