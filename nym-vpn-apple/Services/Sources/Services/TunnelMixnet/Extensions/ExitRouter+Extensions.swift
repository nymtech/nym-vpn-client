#if os(iOS)
import CountriesManager
import MixnetLibrary
import ConnectionTypes

extension ExitRouter {
    var exitPoint: ExitPoint {
        switch self {
        case let .country(country):
            .location(location: country.code)
        case let .gateway(identity):
            .gateway(identity: identity)
        }
    }
}
#endif
