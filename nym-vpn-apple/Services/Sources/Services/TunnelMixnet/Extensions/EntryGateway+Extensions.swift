#if os(iOS)
import CountriesManager
import NymVPNLib
import ConnectionTypes

extension EntryGateway {
    var entryPoint: EntryPoint {
        switch self {
        case let .country(country):
            return .country(twoLetterIsoCountryCode: country.code)
        case let .lowLatencyCountry(country):
            return .country(twoLetterIsoCountryCode: country.code)
        case let .gateway(gateway):
            return .gateway(identity: gateway.id)
        case .random:
            return .random
        }
    }
}
#endif
