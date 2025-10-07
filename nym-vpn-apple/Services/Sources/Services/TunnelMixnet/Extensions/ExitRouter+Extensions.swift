#if os(iOS)
import NymVPNLib
import ConnectionTypes

extension ExitRouter {
    var exitPoint: ExitPoint {
        switch self {
        case let .country(code):
            .country(twoLetterIsoCountryCode: code)
        case let .gateway(gateway):
            .gateway(identity: gateway)
        case .address:
            .random
        case let .region(countryCode: _, region: region):
            .region(region: region)
        case .random:
            .random
        }
    }
}
#endif
