#if os(iOS)
import CountriesManagerTypes
import NymVPNLib

public extension GatewayASNType {
    init(with type: AsnKind) {
        switch type {
        case .residential:
            self = .residential
        case .other:
            self = .other
        }
    }
}
#endif
