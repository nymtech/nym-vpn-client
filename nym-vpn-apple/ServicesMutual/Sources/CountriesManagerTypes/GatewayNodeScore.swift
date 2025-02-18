public enum GatewayNodeScore: Codable {
    case noScore
    case low
    case medium
    case high
    case unrecognized(Int)
}
