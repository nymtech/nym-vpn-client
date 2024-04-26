import CountriesManager
import MixnetLibrary

extension ExitRouter {
    var exitPoint: ExitPoint {
        switch self {
        case .country(let code):
            return .location(location: code)
        case let .lowLatencyCountry(code: code):
            return .location(location: code)
        case .randomLowLatency:
            // TODO: Missing random or something
            return .location(location: "DE")
        }
    }
}
