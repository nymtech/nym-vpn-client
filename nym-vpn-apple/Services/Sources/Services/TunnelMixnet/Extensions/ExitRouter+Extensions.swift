#if os(iOS)
import CountriesManager
import MixnetLibrary

extension ExitRouter {
    var exitPoint: ExitPoint {
        switch self {
        case let .country(country):
            return .location(location: country.code)
        case let .gateway(identity):
            return .gateway(identity: identity)
        }
    }
}
#endif
