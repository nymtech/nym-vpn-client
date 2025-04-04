import SwiftUI
import Theme
import TunnelStatus

public enum ConnectButtonState {
    case connect
    case disconnect
    case disconnecting
    case stop
    case installingDaemon
    case noInternet
    case noInternetReconnect

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
        case .offline, .unknown:
            self = .noInternet
        case .offlineReconnect:
            self = .noInternetReconnect
        }
    }

    public var localizedTitle: String {
        switch self {
        case .connect, .noInternet:
            "connect".localizedString
        case .disconnect:
            "disconnect".localizedString
        case .disconnecting:
            "disconnecting".localizedString
        case .stop, .noInternetReconnect:
            "stop".localizedString
        case .installingDaemon:
            "home.installDaemonButton".localizedString
        }
    }

    var backgroundColor: Color {
        switch self {
        case .connect, .noInternet:
            NymColor.accent
        case .installingDaemon, .noInternetReconnect:
            NymColor.gray1
        case .stop, .disconnecting, .disconnect:
            NymColor.error
        }
    }
}

#if os(macOS)
extension ConnectButtonState {
    public var menuBarItemIsAction: Bool {
        switch self {
        case .connect, .disconnect, .stop, .noInternetReconnect, .noInternet:
            true
        case .disconnecting, .installingDaemon:
            false
        }
    }
}
#endif
