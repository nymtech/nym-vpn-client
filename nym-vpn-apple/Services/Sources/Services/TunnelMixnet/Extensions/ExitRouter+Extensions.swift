#if os(iOS)
import CountriesManager
import MixnetLibrary

extension ExitRouter {
    var exitPoint: ExitPoint {
        switch self {
        case .country(let code):
            return .location(location: code)
        }
    }
}
#endif
