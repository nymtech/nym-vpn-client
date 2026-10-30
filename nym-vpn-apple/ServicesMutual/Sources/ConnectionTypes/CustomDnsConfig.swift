import NymVPNLib

public struct CustomDnsConfig: Codable, Equatable {
    public enum PlainProtocol: String, Codable {
        case udp, tcp, udpAndTcp
    }

    public enum Server: Codable, Hashable {
        case plain(address: String, port: UInt16?, ipProto: PlainProtocol)
        case tls(address: String, port: UInt16?, serverName: String)
        // swiftlint:disable:next enum_case_associated_values_count
        case https(address: String, port: UInt16?, serverName: String, queryPath: String?)
        case quic(address: String, port: UInt16?, serverName: String)
        // swiftlint:disable:next enum_case_associated_values_count
        case http3(address: String, port: UInt16?, serverName: String, queryPath: String?)
    }

    /// Use `dnsServers` instead of the default DNS servers when there is at least one.
    public var isEnabled: Bool
    public var dnsServers: [Server]

    public init(isEnabled: Bool = false, dnsServers: [Server] = []) {
        self.isEnabled = isEnabled
        self.dnsServers = dnsServers
    }

    public init(from settings: CustomDnsSettings) {
        self.isEnabled = settings.enabled
        self.dnsServers = settings.dnsServers.map(Server.init(from:))
    }

    public var settings: CustomDnsSettings {
        CustomDnsSettings(enabled: isEnabled, dnsServers: dnsServers.map(\.dnsServer))
    }
}

extension CustomDnsConfig.PlainProtocol {
    public init(from proto: PlainDnsProtocol) {
        switch proto {
        case .udp:
            self = .udp
        case .tcp:
            self = .tcp
        case .udpAndTcp:
            self = .udpAndTcp
        }
    }

    public var plainDnsProtocol: PlainDnsProtocol {
        switch self {
        case .udp:
            .udp
        case .tcp:
            .tcp
        case .udpAndTcp:
            .udpAndTcp
        }
    }
}

extension CustomDnsConfig.Server {
    public init(from server: DnsServer) {
        switch server {
        case let .plain(address, port, ipProto):
            self = .plain(address: address, port: port, ipProto: .init(from: ipProto))
        case let .tls(address, port, serverName):
            self = .tls(address: address, port: port, serverName: serverName)
        case let .https(address, port, serverName, queryPath):
            self = .https(address: address, port: port, serverName: serverName, queryPath: queryPath)
        case let .quic(address, port, serverName):
            self = .quic(address: address, port: port, serverName: serverName)
        case let .http3(address, port, serverName, queryPath):
            self = .http3(address: address, port: port, serverName: serverName, queryPath: queryPath)
        }
    }

    public var dnsServer: DnsServer {
        switch self {
        case let .plain(address, port, ipProto):
            .plain(address: address, port: port, ipProto: ipProto.plainDnsProtocol)
        case let .tls(address, port, serverName):
            .tls(address: address, port: port, serverName: serverName)
        case let .https(address, port, serverName, queryPath):
            .https(address: address, port: port, serverName: serverName, queryPath: queryPath)
        case let .quic(address, port, serverName):
            .quic(address: address, port: port, serverName: serverName)
        case let .http3(address, port, serverName, queryPath):
            .http3(address: address, port: port, serverName: serverName, queryPath: queryPath)
        }
    }

    public var address: String {
        switch self {
        case let .plain(address, _, _),
             let .tls(address, _, _),
             let .https(address, _, _, _),
             let .quic(address, _, _),
             let .http3(address, _, _, _):
            address
        }
    }

    public var port: UInt16? {
        switch self {
        case let .plain(_, port, _),
             let .tls(_, port, _),
             let .https(_, port, _, _),
             let .quic(_, port, _),
             let .http3(_, port, _, _):
            port
        }
    }

    public var protocolName: String {
        switch self {
        case let .plain(_, _, ipProto):
            switch ipProto {
            case .udp:
                "UDP"
            case .tcp:
                "TCP"
            case .udpAndTcp:
                "UDP/TCP"
            }
        case .tls:
            "TLS"
        case .https:
            "HTTPS"
        case .quic:
            "QUIC"
        case .http3:
            "HTTP/3"
        }
    }

    public var defaultPort: UInt16 {
        switch self {
        case .plain:
            53
        case .tls, .quic:
            853
        case .https, .http3:
            443
        }
    }

    /// Address followed by the port when it differs from the protocol default, e.g. `1.1.1.1:5353` or `[::1]:5353`.
    public var endpoint: String {
        switch port {
        case let .some(port) where port != defaultPort:
            address.contains(":") ? "[\(address)]:\(port)" : "\(address):\(port)"
        default:
            address
        }
    }

    /// Endpoint followed by the protocol, e.g. `9.9.9.9 (TLS)`.
    public var displayName: String {
        "\(endpoint) (\(protocolName))"
    }
}
