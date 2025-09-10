import Foundation
import SwiftUI
import Theme
import TunnelStatus

public enum StatusInfoState: Equatable {
    case initialising
    case connecting(retryAttempt: Int?, tunnelConnectingState: TunnelConnectingState?)
    case connectionTime
    case error(message: String)
    case noInternet
    case noInternetReconnect
    case unknown
    case installingDaemon

    public init(
        tunnelStatus: TunnelStatus,
        isOnline: Bool,
        retryAttempt: Int?,
        tunnelConnectingState: TunnelConnectingState?
    ) {
        switch tunnelStatus {
        case .connected:
            self = .connectionTime
        case .connecting, .reasserting, .restarting:
            self = .connecting(retryAttempt: retryAttempt, tunnelConnectingState: tunnelConnectingState)
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

    var localizedTitle: String {
        switch self {
        case .initialising:
            "initializingClient".localizedString
        case let .connecting(retryAttempt, connectingState):
            switch (retryAttempt, connectingState) {
            case (.none, .none):
                "establishingConnection".localizedString
            case (let .some(retryCount), .none):
                if retryCount > 0 {
                    "\("establishingConnection".localizedString). \("home.connectingRetryAttempt".localizedString): \(retryCount)"
                } else {
                    "establishingConnection".localizedString
                }
            case (.none, let .some(state)):
                "\(state.localizedStringKey.localizedString)"
            case let (.some(retryCount), .some(state)):
                if retryCount > 0 {
                    "\(state.localizedStringKey.localizedString). \("home.connectingRetryAttempt".localizedString): \(retryCount)"
                } else {
                    state.localizedStringKey.localizedString
                }
            }
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

    var textColor: Color {
        switch self {
        case .initialising, .connecting, .connectionTime, .installingDaemon, .noInternet, .noInternetReconnect:
            NymColor.gray1
        case .error, .unknown:
            NymColor.error
        }
    }
}
