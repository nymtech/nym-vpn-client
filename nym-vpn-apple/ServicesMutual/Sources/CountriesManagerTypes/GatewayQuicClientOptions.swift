public struct GatewayQuicClientOptions: Codable, Hashable {
    public var addresses: [String]
    public var host: String?
    public var idPubkey: String

    public init(addresses: [String], host: String?, idPubkey: String) {
        self.addresses = addresses
        self.host = host
        self.idPubkey = idPubkey
    }
}

#if os(iOS)
import NymVPNLib

public extension GatewayQuicClientOptions {
    init(with options: QuicClientOptions) {
        self.init(addresses: options.addresses, host: options.host, idPubkey: options.idPubkey)
    }
}
#elseif os(macOS)
import NymVPNRpc

public extension GatewayQuicClientOptions {
    init(with options: QuicClientOptions) {
        self.init(addresses: options.addresses, host: options.host, idPubkey: options.idPubkey)
    }
}
#endif
