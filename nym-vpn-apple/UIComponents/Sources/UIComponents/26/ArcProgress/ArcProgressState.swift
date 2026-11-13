import Foundation

/// Drives the `ArcProgressView`. Each state corresponds to a discrete visual
/// phase of the connection animation defined by the Figma "Arc Progress"
/// spec sheet (node `1012:22864`).
public enum ArcProgressState: Equatable, Sendable {
    /// Idle / not connected — all rings at track opacity, label "Not protected".
    case disconnected

    /// Device has no internet and no session to resume: idle rings, label
    /// "Not protected" with the no-internet caption.
    case offline

    /// Offline mid-session with the kill switch on, so traffic is held: grey
    /// tracks, only the outer ring lit and blinking, label "Kill switch is
    /// blocking traffic" with the will-reconnect caption.
    case blocked

    /// One of the six connecting steps — fills 50% (first half) or 100% (full)
    /// of the corresponding ring. Outer ring covers steps 1–2, middle 3–4,
    /// inner 5–6.
    case step(Step)

    /// Connection is live — sphere glow on, label fades out.
    case connected

    /// Most recent connect attempt failed — fills shift to error red, sphere
    /// gets the error tint, label "Connection failed".
    case failed

    /// Mid-cancel — fills fade to 15% opacity, no color change.
    case canceling

    /// Gateway independence pre-flight blocked — not connected; label prompts consent.
    case awaitingGatewayConsent

    public enum Step: Equatable, Sendable, CaseIterable {
        case initializingNym
        case authenticatingAccount
        case downloadingZkNyms
        case updatingServerList
        case choosingBestServers
        case registeringWithServers
        case establishingConnection

        /// The marker step (1 to `Marker.connectingStepCount`) this step draws.
        /// Matches `Marker.step(for:)`: the two account checks share step 2.
        public var markerStep: Int {
            switch self {
            case .initializingNym:
                return 1
            case .authenticatingAccount, .downloadingZkNyms:
                return 2
            case .updatingServerList:
                return 3
            case .choosingBestServers:
                return 4
            case .registeringWithServers:
                return 5
            case .establishingConnection:
                return 6
            }
        }
    }
}

/// Visual mode — `fast` is the default mint palette; `anonymous` is the muted
/// gray palette used when the user is connecting through the anonymous pool.
public enum ArcProgressMode: Equatable, Sendable {
    case fast
    case anonymous

    /// Per-step sweep duration used to pace queued ring sweeps.
    public var sweepDuration: Duration {
        switch self {
        case .fast:
            return .milliseconds(800)
        case .anonymous:
            return .milliseconds(1200)
        }
    }
}
