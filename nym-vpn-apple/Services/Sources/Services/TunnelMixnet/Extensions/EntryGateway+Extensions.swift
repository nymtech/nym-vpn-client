#if os(iOS)
import CountriesManager
import MixnetLibrary

extension EntryGateway {
    var entryPoint: EntryPoint {
        switch self {
        case let .country(code):
            return .location(location: code)
        case let .lowLatencyCountry(code):
            return .location(location: code)
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
