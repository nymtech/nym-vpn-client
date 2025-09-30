import Foundation
import CountriesManagerTypes
import Theme

public enum EntryGateway: Codable, Equatable {
    case country(String)
    case region(String)
    case city(String)
    case lowLatencyCountry(String)
    case gateway(String)
    case random

    public var isQuickest: Bool {
        switch self {
        case .country, .random, .gateway, .region, .city:
            false
        case .lowLatencyCountry:
            true
        }
    }

    public var isCountry: Bool {
        switch self {
        case .country:
            true
        case .lowLatencyCountry, .random, .gateway, .region, .city:
            false
        }
    }
}

extension EntryGateway: GatewayInfoProtocol {
    public var countryCode: String? {
        switch self {
        case let .country(code), let .lowLatencyCountry(code):
            code
        case .random, .city, .region, .gateway:
            nil
        }
    }

    public var isGateway: Bool {
        switch self {
        case .country, .lowLatencyCountry, .random, .region, .city:
            false
        case .gateway:
            true
        }
    }

    public var gatewayId: String? {
        switch self {
        case let .gateway(identifier):
            identifier
        case .country, .lowLatencyCountry, .random, .region, .city:
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
