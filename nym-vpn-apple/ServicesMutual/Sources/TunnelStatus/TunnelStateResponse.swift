import Foundation
import ErrorReason

/// Mirror of the core `TunnelState` for the app, which can't link the core library.
public enum TunnelStateResponse: Codable {
    case disconnected
    case connecting(retryAttempt: Int?, state: TunnelConnectingState?, connectionInfo: ConnectionInfoData?)
    case connected(connectionInfo: ConnectionInfoData?, connectedAt: Date?)
    case disconnecting(afterDisconnect: AfterDisconnectAction?)
    case error(ErrorReason)
    case offline(reconnect: Bool)
}

public extension TunnelStateResponse {
    var retryAttempt: Int? {
        if case let .connecting(retryAttempt, _, _) = self { retryAttempt } else { nil }
    }

    var tunnelConnectingState: TunnelConnectingState? {
        if case let .connecting(_, state, _) = self { state } else { nil }
    }

    var connectionInfoData: ConnectionInfoData? {
        switch self {
        case let .connecting(_, _, connectionInfo), let .connected(connectionInfo, _):
            connectionInfo
        default:
            nil
        }
    }

    var connectedAt: Date? {
        if case let .connected(_, connectedAt) = self { connectedAt } else { nil }
    }

    var afterDisconnectAction: AfterDisconnectAction? {
        if case let .disconnecting(afterDisconnect) = self { afterDisconnect } else { nil }
    }
}

public extension TunnelStatus {
    init(_ state: TunnelStateResponse) {
        switch state {
        case .disconnected:
            self = .disconnected
        case .connecting:
            self = .connecting
        case .connected:
            self = .connected
        case .disconnecting:
            self = .disconnecting
        case .error:
            self = .error
        case let .offline(reconnect):
            self = reconnect ? .offlineReconnect : .offline
        }
    }
}
