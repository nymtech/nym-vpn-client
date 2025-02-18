public enum GatewayNodeScore: Codable {
    case none
    case low
    case medium
    case high
    case unrecognized(Int)
}
