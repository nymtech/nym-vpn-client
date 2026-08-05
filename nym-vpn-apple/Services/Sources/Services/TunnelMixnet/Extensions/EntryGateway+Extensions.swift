#if os(iOS)
import NymVPNLib
import ConnectionTypes

extension EntryGateway {
    public var entryPoint: EntryPoint {
        switch self {
        case let .country(code):
            .country(twoLetterIsoCountryCode: code)
        case let .gateway(gateway):
            .gateway(identity: gateway)
        case .random:
            .random
        case .auto:
            .auto(excludeUserCountry: true)
        case let .region(countryCode: _, region: region):
            .region(region: region)
        }
    }
}
#endif
