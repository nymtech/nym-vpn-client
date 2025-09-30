#if os(iOS)
import NymVPNLib
import ConnectionTypes

extension EntryGateway {
    var entryPoint: EntryPoint {
        switch self {
        case let .country(code):
            .location(location: code)
        case let .lowLatencyCountry(code):
            .location(location: code)
        case let .gateway(gateway):
            .gateway(identity: gateway)
        case .random:
            .random
        case let .region(region):
            .random
        case let .city(city):
            .random
        }
    }
}
#endif
