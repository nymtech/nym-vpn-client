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

    /// AmneziaWG version on the entry hop, if any.
    var amneziaVersion: String? {
        guard case let .wireguard(data) = self, let amnezia = data.amnezia else { return nil }
        switch amnezia.protocol {
        case .awg10:
            return "1.0"
        case .awg31:
            return "3.1"
        }
    }

    /// Whether AmneziaWG goes through the gateway's bridge translator or straight to the node.
    var amneziaViaTranslator: Bool? {
        guard case let .wireguard(data) = self else { return nil }
        return data.amnezia?.viaTranslator
    }
}
