#if os(iOS)
import NymVPNLib
import ConnectionTypes

extension ExitRouter {
    public var exitPoint: ExitPoint {
        switch self {
        case let .country(code):
            .country(twoLetterIsoCountryCode: code)
        case let .gateway(gateway):
            .gateway(identity: gateway)
        case let .region(countryCode: _, region: region):
            .region(region: region)
        case .random:
            .random
        case .auto:
            .auto(excludeEntryPointCountry: true, excludeUserCountry: true)
        }
    }
}
#endif
