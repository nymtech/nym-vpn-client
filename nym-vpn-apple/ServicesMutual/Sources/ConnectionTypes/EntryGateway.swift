import Foundation
import CountriesManagerTypes
import Theme

public enum EntryGateway: Codable, Equatable {
    case country(Country)
    case lowLatencyCountry(Country)
    case gateway(GatewayNode)
    case randomLowLatency
    case random

    public var isQuickest: Bool {
        switch self {
        case .country, .random, .gateway:
            false
        case .randomLowLatency, .lowLatencyCountry:
            true
        }
    }

    public var isCountry: Bool {
        switch self {
        case .country:
            true
        case .lowLatencyCountry, .randomLowLatency, .random, .gateway:
            false
        }
    }
}

extension EntryGateway: GatewayInfoProtocol {
    public var name: String {
        switch self {
        case let .country(country), let .lowLatencyCountry(country):
            country.name
        case .randomLowLatency:
            "gateway.randomLowLatency".localizedString
        case .random:
            "gateway.random".localizedString
        case let .gateway(gateway):
            gateway.moniker ?? gateway.id
        }
    }

    public var countryCode: String? {
        switch self {
        case let .country(country), let .lowLatencyCountry(country):
            country.code
        case let .gateway(gateway):
            gateway.countryCode
        case .randomLowLatency, .random:
            nil
        }
    }

    public var isGateway: Bool {
        switch self {
        case .country, .lowLatencyCountry, .randomLowLatency, .random:
            false
        case .gateway:
            true
        }
    }

    public var gatewayId: String? {
        switch self {
        case let .gateway(gateway):
            gateway.id
        case .country, .lowLatencyCountry, .randomLowLatency, .random:
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
