import Foundation

public enum PacketTunnelProviderError: String, Error {
    case invalidSavedConfiguration
    case backendStartFailure
    case noCredentialDataDir
    case startAccountController
}
