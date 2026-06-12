import Foundation

/// Observable phases emitted during account setup for UI copy and carousel sync.
public enum AccountSetupPhase: String, Sendable {
    case idle
    case syncingSummary
    case registeringDevice
    case fetchingTickets
    case ready
}
