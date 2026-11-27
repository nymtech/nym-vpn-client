import Foundation

/// Descriptions omit URLs and PINs, so states can be logged.
public enum PairingState: Equatable, Sendable, CustomStringConvertible {
    case idle
    case startingCode
    /// `expiresAt` is an estimate; the core's `timedOut` event is authoritative.
    case showingCode(url: String, sessionId: UInt32, expiresAt: ContinuousClock.Instant)
    case confirmCode(pin: String, sessionId: UInt32)
    case sharingAccount
    case accountShared
    /// The core sends the PIN once the other device confirms it got the same one.
    case waitingForCode(sessionId: UInt32)
    case waitingForApproval(pin: String, sessionId: UInt32)
    case signedIn
    case expired
    case failed(PairingError)

    public var description: String {
        switch self {
        case .idle:
            "idle"
        case .startingCode:
            "startingCode"
        case let .showingCode(_, sessionId, _):
            "showingCode(sessionId: \(sessionId))"
        case let .confirmCode(_, sessionId):
            "confirmCode(sessionId: \(sessionId))"
        case .sharingAccount:
            "sharingAccount"
        case .accountShared:
            "accountShared"
        case let .waitingForCode(sessionId):
            "waitingForCode(sessionId: \(sessionId))"
        case let .waitingForApproval(_, sessionId):
            "waitingForApproval(sessionId: \(sessionId))"
        case .signedIn:
            "signedIn"
        case .expired:
            "expired"
        case let .failed(error):
            "failed(\(error))"
        }
    }

    var activeSessionId: UInt32? {
        switch self {
        case let .showingCode(_, sessionId, _),
             let .confirmCode(_, sessionId),
             let .waitingForCode(sessionId),
             let .waitingForApproval(_, sessionId):
            sessionId
        default:
            nil
        }
    }
}
