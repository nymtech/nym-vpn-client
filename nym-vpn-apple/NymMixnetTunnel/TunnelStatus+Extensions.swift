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
        case let .disconnecting(actionAfterDisconnect):
            switch actionAfterDisconnect {
            case .nothing, .error:
                self = .disconnecting
            case .reconnect:
                self = .connecting
            case .offline:
                self = .offlineRecconnect
            }
            self = .disconnecting
        case .error:
            self = .disconnected
        case let .offline(reconnect):
            self = reconnect ? .offlineRecconnect : .offline
        }
    }
}
