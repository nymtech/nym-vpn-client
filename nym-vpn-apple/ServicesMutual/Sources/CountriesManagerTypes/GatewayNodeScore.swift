public enum GatewayNodeScore: Codable {
    case noScore
    case low
    case medium
    case high
    case offline

    public var imageName: String {
        switch self {
        case .low, .noScore:
            return "scoreLow"
        case .medium:
            return "scoreMedium"
        case .high:
            return "scoreHigh"
        case .offline:
            return "scoreLow"
        }
    }

    public var localizedKey: String {
        switch self {
        case .noScore:
            "noScore"
        case .low:
            "low"
        case .medium:
            "medium"
        case .high:
            "high"
        case .offline:
            "offline"
        }
    }
}
