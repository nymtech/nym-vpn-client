import Foundation
import ConnectionTypes

public enum TunnelProviderMessage: Codable {
    case status
    case setCustomDns([String])
    case setEnableCustomDns(Bool)
    case setEnableTwoHop(Bool)
    case setEnableAdBlocking(Bool)
    case setEnableBridges(Bool)
    case setEntryPoint(EntryGateway)
    case setExitPoint(ExitRouter)
    case setFrontingModeEnabled(Bool)
    case setDisableIpv6(Bool)
    case setMixnetTrafficConfig(MixnetTuningConfig)
    case setGatewayIndependence(Bool)
    case setGatewayIndependenceNotifications(Bool)

    public init(messageData: Data) throws {
        self = try JSONDecoder().decode(Self.self, from: messageData)
    }

    public func encode() throws -> Data {
        try JSONEncoder().encode(self)
    }
}
