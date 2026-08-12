import NymVPNLib

extension GatewayNodeScore {
    public init(with score: Score?) {
        guard let score else {
            self = .noScore
            return
        }
        switch score {
        case .high:
            self = .high
        case .medium:
            self = .medium
        case .low:
            self = .low
        case .offline:
            self = .offline
        }
    }
}
