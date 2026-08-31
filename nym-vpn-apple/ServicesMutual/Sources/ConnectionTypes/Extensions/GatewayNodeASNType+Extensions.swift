import NymVPNLib

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
