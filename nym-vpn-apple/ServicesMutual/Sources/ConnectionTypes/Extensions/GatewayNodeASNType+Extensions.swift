#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import NymVPNRpc
#endif

public extension GatewayNodeASNType {
    init(with type: AsnKind) {
        switch type {
        case .residential:
            self = .residential
        case .other:
            self = .other
        }
    }
}
