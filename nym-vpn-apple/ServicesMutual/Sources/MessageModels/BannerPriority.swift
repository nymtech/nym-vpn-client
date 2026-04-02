public enum BannerPriority: Int, Comparable, Sendable {
    case low = 0
    case normal = 1
    case high = 2

    public static func < (lhs: BannerPriority, rhs: BannerPriority) -> Bool {
        lhs.rawValue < rhs.rawValue
    }
}
