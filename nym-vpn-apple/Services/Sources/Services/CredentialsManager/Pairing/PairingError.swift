import Foundation

public enum PairingError: Error, Equatable, Sendable {
    case invalidCode
    case alreadySignedIn
    case notSignedIn
    /// The tunnel holds the account store while connected.
    case vpnConnected
    case offline
    case server
    case sessionEnded
    case unknown
}
