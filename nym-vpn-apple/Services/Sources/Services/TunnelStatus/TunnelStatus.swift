import NetworkExtension

public enum TunnelStatus: Int, Equatable {
    case connected
    case connecting
    case disconnected
    case disconnecting
    case reasserting // Not a possible state at present
    case restarting // Restarting tunnel (done after saving modifications to an active tunnel)
    // case waiting    // Waiting for another tunnel to be brought down

    public init(from systemStatus: NEVPNStatus) {
        switch systemStatus {
        case .connected:
            self = .connected
        case .connecting:
            self = .connecting
        case .disconnected:
            self = .disconnected
        case .disconnecting:
            self = .disconnecting
        case .reasserting:
            self = .reasserting
        case .invalid:
            self = .disconnected
        @unknown default:
            fatalError("Unknown TunnelStatus")
        }
    }
}
