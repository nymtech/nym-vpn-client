#if os(macOS)
import NymVPNRpc

extension EntryGateway {
    public var entryPoint: EntryPoint {
        switch self {
        case let .country(code):
            EntryPoint.country(twoLetterIsoCountryCode: code)
        case let .gateway(node):
            EntryPoint.gateway(identity: node)
        case .random:
            EntryPoint.random
        case .auto:
            EntryPoint.auto(excludeUserCountry: true)
        case let .region(countryCode: _, region: region):
            EntryPoint.region(region: region)
        }
    }
}

extension ExitRouter {
    public var exitPoint: ExitPoint {
        switch self {
        case let .country(code):
            ExitPoint.country(twoLetterIsoCountryCode: code)
        case let .gateway(node):
            ExitPoint.gateway(identity: node)
        case let .region(countryCode: _, region: region):
            ExitPoint.region(region: region)
        case .random:
            ExitPoint.random
        case .auto:
            ExitPoint.auto(excludeEntryPointCountry: true, excludeUserCountry: true)
        }
    }
}
#endif
