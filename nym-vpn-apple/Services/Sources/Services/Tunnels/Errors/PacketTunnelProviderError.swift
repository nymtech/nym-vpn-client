import Foundation

public enum PacketTunnelProviderError: String, Error {
    case invalidSavedConfiguration
    case dnsResolveFailure
    case backendStartFailure
    case fileDescriptorFailure
    case saveNetworkSettingsFailure
}
