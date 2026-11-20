import TunnelStatus

/// Connection marker state model (#6909): maps the tunnel status onto the marker's states.
/// Covers five of the six design states; connected with data comes with the data line.
/// Intended to replace `ArcProgressState` once the marker is drawn from it.
public enum MarkerState: Equatable, Sendable {
    case notProtected
    /// step runs 1 to `Marker.connectingStepCount`
    case connecting(step: Int)
    case connected
    case error
    case blocked
    /// Not one of the design states: a short fade out of the current state.
    case stopping
}

/// The face: two blocks for dVPN (2 hops), the mix field for Mixnet.
public enum MarkerMode: Equatable, Sendable {
    case dvpn
    case mixnet
}

public struct Marker: Equatable, Sendable {
    public static let connectingStepCount = 6

    public let state: MarkerState
    public let mode: MarkerMode

    /// Blocked only while traffic is really held: offline mid-session (`.offlineReconnect`)
    /// with the kill switch on. Offline with nothing to resume leaves networking open, so it is
    /// not protected. The Apple apps keep the kill switch always on, so the default stays true.
    public init(
        tunnelStatus: TunnelStatus,
        connectingState: TunnelConnectingState?,
        isMixnet: Bool,
        killSwitch: Bool = true
    ) {
        mode = isMixnet ? .mixnet : .dvpn
        switch tunnelStatus {
        case .disconnected, .unknown, .offline:
            state = .notProtected
        case .connecting, .reasserting, .restarting:
            state = .connecting(step: Marker.step(for: connectingState))
        case .connected:
            state = .connected
        case .disconnecting:
            state = .stopping
        case .error:
            state = .error
        case .offlineReconnect:
            state = killSwitch ? .blocked : .notProtected
        }
    }

    /// Core reports seven connecting phases; the marker draws six steps (three rings,
    /// each half then sealed). The two account checks share step 2.
    public static func step(for connectingState: TunnelConnectingState?) -> Int {
        switch connectingState {
        case .none, .resolvingApiAddresses, .unrecognized:
            1
        case .awaitingAccountReadiness, .awaitingCredentialsAvailability:
            2
        case .refreshingGateways:
            3
        case .selectingGateways:
            4
        case .registeringWithGateways:
            5
        case .connectingTunnel:
            6
        }
    }
}
