import Foundation
import CountriesManagerTypes
import Theme

public enum ExitRouter: Codable, Equatable {
    case country(Country)
    case gateway(GatewayNode)

    public var isCountry: Bool {
        switch self {
        case .country:
            true
        case .gateway:
            false
        }
    }
}

extension ExitRouter: GatewayInfoProtocol {
    public var name: String {
        switch self {
        case let .country(country):
            country.name
        case let .gateway(gateway):
            gateway.moniker ?? gateway.id
        }
    }

    public var countryCode: String? {
        switch self {
        case let .country(country):
            country.code
        case let .gateway(gateway):
            gateway.countryCode
        }
    }

    public var isGateway: Bool {
        switch self {
        case .country:
            false
        case .gateway:
            true
        }
    }

    public var gatewayId: String? {
        switch self {
        case .country:
            nil
        case let .gateway(gateway):
            gateway.id
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
