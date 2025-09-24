#if os(iOS)
import CountriesManager
import NymVPNLib
import ConnectionTypes

extension ExitRouter {
    var exitPoint: ExitPoint {
        switch self {
        case let .country(country):
            .country(twoLetterIsoCountryCode: country.code)
        case let .gateway(gateway):
            .gateway(identity: gateway.id)
        }
    }
}
#endif
