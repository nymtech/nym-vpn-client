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
        }
    }

    public init(hasInternet: Bool) {
        self = hasInternet ? .unknown : .noInternet
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
        case .noInternet:
            "home.deviceNoInternet".localizedString
        case .noInternetReconnect:
            "home.deviceNoInternetReconnect".localizedString
        case .unknown:
            // Empty string hides the view. To not mess up UX spacing - need 'space' to still show it.
            " "
        case .installingDaemon:
            "home.installDaemon".localizedString
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
