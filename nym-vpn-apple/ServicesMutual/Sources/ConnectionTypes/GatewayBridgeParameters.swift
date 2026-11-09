import NymVPNLib

public enum GatewayBridgeParameters: Codable, Hashable {
    case quicPlain(GatewayQuicClientOptions)
    case tlsPlain(GatewayTlsClientOptions)
    case sshPlain(GatewaySshPlainClientOptions)
}

public extension GatewayBridgeParameters {
    /// The AmneziaWG translator is dialed by the daemon, not shown as a bridge transport.
    init?(with parameters: ClientConfig) {
        switch parameters {
        case let .quicPlain(quicClientOptions):
            self = .quicPlain(GatewayQuicClientOptions(with: quicClientOptions))
        case let .tlsPlain(tlsClientOptions):
            self = .tlsPlain(GatewayTlsClientOptions(with: tlsClientOptions))
        case let .sshPlain(sshOptions):
            self = .sshPlain(GatewaySshPlainClientOptions(with: sshOptions))
        case .amnezia:
            return nil
        }
    }
}
