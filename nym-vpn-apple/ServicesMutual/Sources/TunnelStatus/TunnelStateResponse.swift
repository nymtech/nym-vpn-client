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
