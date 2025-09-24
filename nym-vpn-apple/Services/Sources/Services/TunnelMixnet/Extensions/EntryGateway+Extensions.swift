#if os(iOS)
import CountriesManager
import NymVPNLib
import ConnectionTypes

extension EntryGateway {
    var entryPoint: EntryPoint {
        switch self {
        case let .country(country):
            return .location(location: country.code)
        case let .lowLatencyCountry(country):
            return .location(location: country.code)
        case let .gateway(gateway):
            return .gateway(identity: gateway.id)
        case .random:
            return .random
        }
    }
}
#endif
