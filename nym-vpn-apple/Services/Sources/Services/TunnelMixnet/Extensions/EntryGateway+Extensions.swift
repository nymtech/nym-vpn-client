#if os(iOS)
import NymVPNLib
import ConnectionTypes

extension EntryGateway {
    var entryPoint: EntryPoint {
        switch self {
        case let .country(country):
            .location(location: country.code)
        case let .lowLatencyCountry(country):
            .location(location: country.code)
        case let .gateway(gateway):
            .gateway(identity: gateway.id)
        case .random:
            .random
        }
    }
}
#endif
