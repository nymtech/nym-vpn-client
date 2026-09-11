import NymVPNLib

public struct GatewaySshPlainClientOptions: Hashable, Codable {
    public var addresses: [BridgeSocketAddr]
    public var idPubkey: String
    public var username: String?
    public var clientAuthKey: String
    public var clientBanner: String?
}

public extension GatewaySshPlainClientOptions {
    init(with options: SshPlainClientOptions) {
        self.init(
            addresses: options.addresses,
            idPubkey: options.idPubkey,
            username: options.username,
            clientAuthKey: options.clientAuthKey,
            clientBanner: options.clientBanner
        )
    }
}
