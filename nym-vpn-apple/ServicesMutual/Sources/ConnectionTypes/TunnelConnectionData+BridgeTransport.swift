import NymVPNLib

public extension TunnelConnectionData {
    var bridgeTransportName: String? {
        guard case let .wireguard(data) = self, let transport = data.entryBridgeAddr?.transport else { return nil }
        switch transport {
        case .quic:
            return "QUIC"
        case .tls:
            return "TLS"
        case .ssh:
            return "SSH"
        }
    }
}
