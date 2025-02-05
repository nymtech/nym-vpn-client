import MixnetLibrary
import TunnelStatus

extension TunnelStatus {
    init(from tunnelState: TunnelState) {
        switch tunnelState {
        case .disconnected:
            self = .disconnected
        case .connecting:
            self = .connecting
        case .connected:
            self = .connected
        case .disconnecting:
            self = .disconnecting
        case .error:
            self = .disconnected
        case let .offline(reconnect):
            self = reconnect ? .offlineReconnect : .offline
        }
    }
}
