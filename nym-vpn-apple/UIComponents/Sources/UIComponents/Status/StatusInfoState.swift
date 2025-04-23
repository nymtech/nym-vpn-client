import Foundation
import SwiftUI
import Theme
import TunnelStatus

public enum StatusInfoState: Equatable {
    case initialising
    case connecting
    case connectionTime
    case error(message: String)
    case noInternet
    case noInternetReconnect
    case unknown
    case installingDaemon

    public init(tunnelStatus: TunnelStatus, isOnline: Bool) {
        switch tunnelStatus {
        case .connected:
            self = .connectionTime
        case .connecting, .reasserting, .restarting:
            self = .connecting
        case .disconnected, .disconnecting, .unknown:
            self = isOnline ? .unknown : .noInternet
        case .offline:
            self = .noInternet
        case .offlineReconnect:
            self = .noInternetReconnect
        case .error:
            self = .error(message: " ")
        }
    }

    public init(hasInternet: Bool) {
        self = hasInternet ? .unknown : .noInternet
    }

    var titleLocalizedKey: String {
        switch self {
        case .initialising:
            "initializingClient"
        case .connecting:
            "establishingConnection"
        case .connectionTime:
            "connectionTime"
        case let .error(message):
            message
        case .noInternet:
            "home.deviceNoInternet"
        case .noInternetReconnect:
            "home.deviceNoInternetReconnect"
        case .unknown:
            // Empty string hides the view. To not mess up UX spacing - need 'space' to still show it.
            " "
        case .installingDaemon:
            "home.installDaemon"
        }
    }

    var isConnecting: Bool {
        self == .connecting
    }

    var textColor: Color {
        switch self {
        case .initialising, .connecting, .connectionTime, .installingDaemon, .noInternet, .noInternetReconnect:
            NymColor.gray1
        case .error, .unknown:
            NymColor.error
        }
    }
}
