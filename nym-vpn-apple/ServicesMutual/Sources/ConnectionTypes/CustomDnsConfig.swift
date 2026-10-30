import NymVPNLib

public struct CustomDnsConfig: Codable, Equatable {
    public enum PlainProtocol: String, Codable {
        case udp, tcp, udpAndTcp
    }

    public enum Server: Codable, Equatable {
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
}
