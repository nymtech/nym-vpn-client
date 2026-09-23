import NymVPNLib

public enum GatewayBridgeParameters: Codable, Hashable {
    case quicPlain(GatewayQuicClientOptions)
    case tlsPlain(GatewayTlsClientOptions)
    case sshPlain(GatewaySshPlainClientOptions)
}

public extension GatewayBridgeParameters {
    init(with parameters: ClientConfig) {
        switch parameters {
        case let .quicPlain(quicClientOptions):
            self = .quicPlain(GatewayQuicClientOptions(with: quicClientOptions))
        case let .tlsPlain(tlsClientOptions):
            self = .tlsPlain(GatewayTlsClientOptions(with: tlsClientOptions))
        case let .sshPlain(sshOptions):
            self = .sshPlain(GatewaySshPlainClientOptions(with: sshOptions))
        }
    }
}
