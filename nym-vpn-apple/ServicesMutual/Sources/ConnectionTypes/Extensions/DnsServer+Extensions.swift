import NymVPNLib

extension CustomDnsSettings {
    /// Plain DNS over UDP and TCP on the default port for each address.
    public init(plainDnsAddresses: [IpAddr]) {
        self.init(dnsServers: plainDnsAddresses.map { .plain(address: $0, port: nil, ipProto: .udpAndTcp) })
    }
}

extension DnsServer {
    public var address: IpAddr {
        switch self {
        case let .plain(address, _, _),
             let .tls(address, _, _),
             let .https(address, _, _, _),
             let .quic(address, _, _),
             let .http3(address, _, _, _):
            address
        }
    }
}
