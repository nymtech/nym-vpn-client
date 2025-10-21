public enum GatewayBridgeParameters: Codable, Hashable {
    case quicPlain(GatewayQuicClientOptions)
}

#if os(iOS)
import NymVPNLib

public extension GatewayBridgeParameters {
    init(with parameters: BridgeParameters) {
        switch parameters {
        case let .quicPlain(quicClientOptions):
            self = .quicPlain(GatewayQuicClientOptions(with: quicClientOptions))
        }
    }
}
#elseif os(macOS)
import NymVPNRpc

public extension GatewayBridgeParameters {
    init(with parameters: BridgeParameters) {
        switch parameters {
        case let .quicPlain(quicClientOptions):
            self = .quicPlain(GatewayQuicClientOptions(with: quicClientOptions))
        }
    }
}
#endif
