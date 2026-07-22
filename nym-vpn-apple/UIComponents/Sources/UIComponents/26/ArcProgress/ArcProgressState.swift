import Foundation

/// Drives the `ArcProgressView`. Each state corresponds to a discrete visual
/// phase of the connection animation defined by the Figma "Arc Progress"
/// spec sheet (node `1012:22864`).
public enum ArcProgressState: Equatable, Sendable {
    /// Idle / not connected — all rings at track opacity, label "Not protected".
    case disconnected

    /// Device has no internet — idle rings, label "Your device has no internet connection".
    case offline

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
        case .fast: .milliseconds(800)
        case .anonymous: .milliseconds(1200)
        }
    }
}
