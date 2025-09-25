import CountriesManagerTypes

extension GatewayASNType {
    init(with type: NymVpnService_AsnKind) {
        switch type {
        case .residential:
            self = .residential
        case .other:
            self = .other
        case .UNRECOGNIZED:
            self = .other
        }
    }
}
