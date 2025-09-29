import Foundation
import CountriesManagerTypes
import Theme

public enum EntryGateway: Codable, Equatable {
    case country(Country)
    case lowLatencyCountry(Country)
    case gateway(GatewayNode)
    case random

    public var isQuickest: Bool {
        switch self {
        case .country, .random, .gateway:
            false
        case .lowLatencyCountry:
            true
        }
    }

    public var isCountry: Bool {
        switch self {
        case .country:
            true
        case .lowLatencyCountry, .random, .gateway:
            false
        }
    }
}

extension EntryGateway: GatewayInfoProtocol {
    // Returns moniker or country code
    public var name: String {
        switch self {
        case let .country(country), let .lowLatencyCountry(country):
            country.code
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
            gateway.location.twoLetterIsoCountryCode
        case .random:
            nil
        }
    }

    public var isGateway: Bool {
        switch self {
        case .country, .lowLatencyCountry, .random:
            false
        case .gateway:
            true
        }
    }

    public var gatewayId: String? {
        switch self {
        case let .gateway(gateway):
            gateway.id
        case .country, .lowLatencyCountry, .random:
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
