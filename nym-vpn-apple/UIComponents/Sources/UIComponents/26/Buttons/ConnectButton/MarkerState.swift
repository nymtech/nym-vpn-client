import TunnelStatus

/// Connection marker, state model (#7063, PR 1 of 5).
/// Maps the tunnel status onto the marker's six visual states. No view reads this yet.
public enum MarkerState: Equatable {
    case notProtected
    /// step runs 1 to `Marker.connectingStepCount`
    case connecting(step: Int)
    case connected
    case error
    case blocked(BlockedCause)
    /// Not one of the six Figma variants: a short fade out of the current state.
    case stopping
}

/// The face: two blocks for dVPN (2 hops), the 3x3 mix field for Mixnet.
public enum MarkerMode: Equatable {
    case dvpn
    case mixnet
}

/// Why traffic is blocked, so the copy never promises a fix that can't work.
public enum BlockedCause: Equatable {
    case needsReconnect
    case noInternet
}

public struct Marker: Equatable {
    public static let connectingStepCount = 6

    public let state: MarkerState
    public let mode: MarkerMode

    /// Kill switch on = Blocked, off = Not protected (Hux, 2026-11-13).
    /// The Apple apps keep the kill switch always on, so the default stays true.
    public init(
        tunnelStatus: TunnelStatus,
        connectingState: TunnelConnectingState?,
        isMixnet: Bool,
        killSwitch: Bool = true
    ) {
        mode = isMixnet ? .mixnet : .dvpn
        switch tunnelStatus {
        case .disconnected, .unknown:
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
            state = killSwitch ? .blocked(.needsReconnect) : .notProtected
        case .offline:
            state = killSwitch ? .blocked(.noInternet) : .notProtected
        }
    }

    /// Core reports seven connecting phases; the marker draws six steps (three rings,
    /// each half then sealed). The two account checks are one step: "Checking your account".
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
