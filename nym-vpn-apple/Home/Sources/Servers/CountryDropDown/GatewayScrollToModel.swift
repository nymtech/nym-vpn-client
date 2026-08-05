import ConnectionTypes
import ConnectionTypes

public enum GatewayScrollToModel: Equatable {
    case country(code: String)
    case region(countryCode: String, region: String)
    case server(id: String)
    case empty

    public init(entryGateaway: EntryGateway) {
        switch entryGateaway {
        case let .country(code):
            self = .country(code: code)
        case let .region(countryCode, region):
            self = .region(countryCode: countryCode, region: region)
        case let .gateway(identifier):
            self = .server(id: identifier)
        case .random, .auto:
            self = .empty
        }
    }

    public init(exitRouter: ExitRouter) {
        switch exitRouter {
        case let .country(code):
            self = .country(code: code)
        case let .region(countryCode, region):
            self = .region(countryCode: countryCode, region: region)
        case let .gateway(identifier):
            self = .server(id: identifier)
        case .random, .auto:
            self = .empty
        }
    }

    var scrollToIdentifier: String {
        switch self {
        case let .country(code):
            return "country:\(code)"
        case let .region(countryCode, region):
            return "region:\(countryCode)_\(region)"
        case let .server(id):
            return "server:\(id)"
        case .empty:
            return "empty"
        }
    }

    var countryCode: String? {
        switch self {
        case let .country(code), let .region(code, _):
            return code
        default:
            return nil
        }
    }

    var isCountry: Bool {
        switch self {
        case .country:
            true
        case .region, .server, .empty:
            false
        }
    }

    var region: String? {
        switch self {
        case let .region(_, code):
            code
        case .country, .server, .empty:
            nil
        }
    }

    var isRegion: Bool {
        switch self {
        case .region:
            true
        case .country, .server, .empty:
            false
        }
    }

    var serverId: String? {
        switch self {
        case let .server(identifier):
            identifier
        case .empty, .region, .country:
            nil
        }
    }

    var isServer: Bool {
        switch self {
        case .server:
            true
        case .country, .region, .empty:
            false
        }
    }

    func shouldExpand(countryCode: String, region: String?, server: GatewayNode?) -> Bool {
        switch self {
        case .country:
            return false
        case let .region(regionCountryCode, _):
            if region != nil {
                return false
            } else {
                return countryCode == regionCountryCode
            }
        case let .server(id):
            return server?.id == id && server?.location?.twoLetterIsoCountryCode == countryCode
            || server?.id == id && server?.location?.region == region
        case .empty:
            return false
        }
    }
}
