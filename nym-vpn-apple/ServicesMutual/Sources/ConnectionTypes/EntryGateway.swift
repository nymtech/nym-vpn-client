import Foundation
import Theme

public enum EntryGateway: Codable, Equatable {
    case country(String)
    case region(countryCode: String, region: String)
    case gateway(String)
    case random
    case auto

    public var isCountry: Bool {
        switch self {
        case .country:
            true
        case .random, .auto, .gateway, .region:
            false
        }
    }

    public var isRegion: Bool {
        switch self {
        case .region:
            true
        case .gateway, .random, .auto, .country:
            false
        }
    }
}

extension EntryGateway: GatewayInfoProtocol {
    public var countryCode: String? {
        switch self {
        case let .country(code):
            code
        case .random, .auto, .region, .gateway:
            nil
        }
    }

    public var isGateway: Bool {
        switch self {
        case .country, .random, .auto, .region:
            false
        case .gateway:
            true
        }
    }

    public var gatewayId: String? {
        switch self {
        case let .gateway(identifier):
            identifier
        case .country, .random, .auto, .region:
            nil
        }
    }
}

extension EntryGateway {
    public func toJson() -> String? {
        guard let jsonData = try? JSONEncoder().encode(self) else { return nil }
        return String(data: jsonData, encoding: .utf8)
    }

    public static func from(jsonString: String) -> EntryGateway? {
        guard let jsonData = jsonString.data(using: .utf8) else { return nil }
        return try? JSONDecoder().decode(EntryGateway.self, from: jsonData)
    }
}
