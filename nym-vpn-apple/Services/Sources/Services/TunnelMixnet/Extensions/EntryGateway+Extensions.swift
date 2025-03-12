#if os(iOS)
import CountriesManager
import MixnetLibrary
import ConnectionTypes

extension EntryGateway {
    var entryPoint: EntryPoint {
        switch self {
        case let .country(country):
            return .location(location: country.code)
        case let .lowLatencyCountry(country):
            return .location(location: country.code)
        case .randomLowLatency:
            return .randomLowLatency
        case let .gateway(gateway):
            return .gateway(identity: gateway.id)
        case .random:
            return .random
        }
    }
}
#endif
