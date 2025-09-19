import CountriesManagerTypes

extension GatewayNodeScore {
    init(with score: NymVpnService_Score) {
        switch score {
        case .low:
            self = .low
        case .medium:
            self = .medium
        case .high:
            self = .high
        case .offline:
            self = .offline
        case .UNRECOGNIZED:
            self = .noScore
        }
    }
}
