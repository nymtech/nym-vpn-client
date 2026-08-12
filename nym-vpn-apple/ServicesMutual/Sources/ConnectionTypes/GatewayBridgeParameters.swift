import NymVPNLib

public enum GatewayBridgeParameters: Codable, Hashable {
    case quicPlain(GatewayQuicClientOptions)
    case tlsPlain(GatewayTlsClientOptions)
}

public extension GatewayBridgeParameters {
    init(with parameters: ClientConfig) {
        switch parameters {
        case let .quicPlain(quicClientOptions):
            self = .quicPlain(GatewayQuicClientOptions(with: quicClientOptions))
        case let .tlsPlain(tlsClientOptions):
            self = .tlsPlain(GatewayTlsClientOptions(with: tlsClientOptions))
        }
    }
}
