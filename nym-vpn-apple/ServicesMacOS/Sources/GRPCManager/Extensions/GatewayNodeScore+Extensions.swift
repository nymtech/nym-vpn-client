import CountriesManagerTypes

extension GatewayNodeScore {
    init(with score: Nym_Vpn_Score) {
        switch score {
        case .none:
            self = .noScore
        case .low:
            self = .low
        case .medium:
            self = .medium
        case .high:
            self = .high
        case let .UNRECOGNIZED(number):
            self = .unrecognized(number)
        }
    }
}
