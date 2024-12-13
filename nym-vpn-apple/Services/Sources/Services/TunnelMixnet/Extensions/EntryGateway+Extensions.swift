#if os(iOS)
import CountriesManager
import MixnetLibrary

extension EntryGateway {
    var entryPoint: EntryPoint {
        switch self {
        case let .country(country):
            return .location(location: country.code)
        case let .lowLatencyCountry(country):
            return .location(location: country.code)
        case .randomLowLatency:
            return .randomLowLatency
        case let .gateway(identity):
            return .gateway(identity: identity)
        case .random:
            return .random
        }
    }
}
#endif
