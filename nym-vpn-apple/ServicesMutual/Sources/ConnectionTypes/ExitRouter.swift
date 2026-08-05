import Foundation
import Theme

public enum ExitRouter: Codable, Equatable {
    case country(String)
    case gateway(String)
    case region(countryCode: String, region: String)
    case random
    case auto

    public var isCountry: Bool {
        switch self {
        case .country:
            true
        case .gateway, .random, .auto, .region:
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

extension ExitRouter: GatewayInfoProtocol {
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
        case .country:
            false
        case .gateway, .random, .auto, .region:
            true
        }
    }

    public var gatewayId: String? {
        switch self {
        case .country, .random, .auto, .region:
            nil
        case let .gateway(gateway):
            gateway
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
