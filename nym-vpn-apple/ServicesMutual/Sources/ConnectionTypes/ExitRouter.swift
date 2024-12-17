import Foundation
import CountriesManagerTypes
import Theme

public enum ExitRouter: Codable, Equatable {
    case country(Country)
    case gateway(String)
    case random

    public var isCountry: Bool {
        switch self {
        case .country:
            true
        case .gateway, .random:
            false
        }
    }
}

extension ExitRouter: GatewayInfoProtocol {
    public var name: String {
        switch self {
        case let .country(country):
            country.name
        case let .gateway(identifier):
            identifier
        case .random:
            "random".localizedString
        }
    }

    public var countryCode: String? {
        switch self {
        case let .country(country):
            country.code
        case .gateway, .random:
            nil
        }
    }

    public var isGateway: Bool {
        switch self {
        case .country, .random:
            false
        case .gateway:
            true
        }
    }
}

extension ExitRouter {
    public func toJson() -> String? {
        guard let jsonData = try? JSONEncoder().encode(self) else { return nil }
        return String(data: jsonData, encoding: .utf8)
    }

    public static func from(jsonString: String) -> ExitRouter? {
        guard let jsonData = jsonString.data(using: .utf8) else { return nil }
        return try? JSONDecoder().decode(ExitRouter.self, from: jsonData)
    }
}
