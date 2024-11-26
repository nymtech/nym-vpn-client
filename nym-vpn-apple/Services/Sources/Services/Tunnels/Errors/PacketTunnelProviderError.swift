import Foundation

public enum PacketTunnelProviderError: String, Error {
    case invalidSavedConfiguration
    case backendStartFailure
    case noCredentialDataDir
    case startAccountController

    /// Tunnel is cancelled because state machine entered error state.
    case errorState
}
