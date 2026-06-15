import Foundation

public final class GatewayNodeStore: Codable {
    public typealias RawValue = String

    public var entry: [GatewayNode]
    public var exit: [GatewayNode]
    public var vpn: [GatewayNode]
    public var lastFetchDate: Date?
#if SANTA
    public var fetchedForEnv: String?
#endif

    public init(
        lastFetchDate: Date? = nil,
#if SANTA
        fetchedForEnv: String? = nil,
#endif
        entry: [GatewayNode] = [],
        exit: [GatewayNode] = [],
        vpn: [GatewayNode] = []
    ) {
        self.lastFetchDate = lastFetchDate
#if SANTA
        self.fetchedForEnv = fetchedForEnv
#endif
        self.entry = entry
        self.exit = exit
        self.vpn = vpn
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(entry, forKey: .entryNodes)
        try container.encode(exit, forKey: .exitNodes)
        try container.encode(vpn, forKey: .vpnNodes)

        if let lastFetchDate = lastFetchDate {
            try container.encode(lastFetchDate.timeIntervalSince1970, forKey: .lastFetchDate)
        }
#if SANTA
        if let fetchedForEnv = fetchedForEnv {
            try container.encode(fetchedForEnv, forKey: .fetchedForEnv)
        }
#endif
    }

    public required init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        entry = try container.decode([GatewayNode].self, forKey: .entryNodes)
        exit = try container.decode([GatewayNode].self, forKey: .exitNodes)
        vpn = try container.decode([GatewayNode].self, forKey: .vpnNodes)

        if let timeInterval = try? container.decode(Double.self, forKey: .lastFetchDate) {
            lastFetchDate = Date(timeIntervalSince1970: timeInterval)
        } else {
            lastFetchDate = nil
        }
#if SANTA
        fetchedForEnv = try container.decodeIfPresent(String.self, forKey: .fetchedForEnv)
#endif
    }

    public var rawValue: RawValue {
        guard let data = try? JSONEncoder().encode(self),
              let result = String(data: data, encoding: .utf8)
        else {
            return ""
        }
        return result
    }

    public convenience init?(rawValue: RawValue) {
        guard let data = rawValue.data(using: .utf8),
              let gatewayNodeStore = try? JSONDecoder().decode(GatewayNodeStore.self, from: data)
        else {
            return nil
        }
        self.init(
            lastFetchDate: gatewayNodeStore.lastFetchDate,
#if SANTA
            fetchedForEnv: gatewayNodeStore.fetchedForEnv,
#endif
            entry: gatewayNodeStore.entry,
            exit: gatewayNodeStore.exit,
            vpn: gatewayNodeStore.vpn
        )
    }

    private enum CodingKeys: String, CodingKey {
        case entryNodes
        case exitNodes
        case vpnNodes
        case lastFetchDate
#if SANTA
        case fetchedForEnv
#endif
    }
}
