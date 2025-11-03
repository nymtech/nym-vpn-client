public enum GatewayNodeScore: Int, Codable {
    case high = 0
    case medium = 1
    case low = 2
    case offline = 3
    case noScore = 4

    public var imageName: String {
        switch self {
        case .low:
            return "scoreLow"
        case .medium:
            return "scoreMedium"
        case .high:
            return "scoreHigh"
        case .offline, .noScore:
            return "scoreOffline"
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
