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
    case subscriptionExpired

    public init(tunnelStatus: TunnelStatus, hasInternet: Bool, subscriptionDidExpire: Bool = false) {
        if !hasInternet {
            self = .noInternet
            return
        }
        if subscriptionDidExpire {
            self = .subscriptionExpired
            return
        }
        switch tunnelStatus {
        case .connected:
            self = .connected
        case .connecting, .reasserting, .restarting:
            self = .connecting
        case .disconnected, .unknown:
            self = .disconnected
        case .disconnecting:
            self = .disconnecting
        case .offline:
            self = .noInternet
        case .offlineReconnect:
            self = .noInternetReconnect
        case .error:
            self = .error
        }
    }

    public var title: String {
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
        case .error, .subscriptionExpired:
            NymColor.black
        }
    }

    var backgroundColor: Color {
        switch self {
        case .connected:
            NymColor.action.opacity(0.1)
        case .connecting, .disconnecting, .disconnected:
            NymColor.backgroundHover
        case .noInternet, .noInternetReconnect, .error, .subscriptionExpired:
            NymColor.error
        }
    }
}
