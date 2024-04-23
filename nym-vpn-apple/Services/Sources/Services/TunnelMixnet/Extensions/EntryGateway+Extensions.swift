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
        case .random:
            return .random
        }
    }
}
