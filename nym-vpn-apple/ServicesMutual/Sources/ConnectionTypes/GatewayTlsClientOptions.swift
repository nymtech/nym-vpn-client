import NymVPNLib

public struct GatewayTlsClientOptions: Codable, Hashable {
    public var addresses: [String]
    public var host: String?
    public var idPubkey: String

    public init(addresses: [String], host: String?, idPubkey: String) {
        self.addresses = addresses
        self.host = host
        self.idPubkey = idPubkey
    }
}

public extension GatewayTlsClientOptions {
    init(with options: TlsPlainClientOptions) {
        self.init(addresses: options.addresses, host: options.host, idPubkey: options.idPubkey)
    }
}
