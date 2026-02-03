public enum ContinuousTraffic: Int, CaseIterable, Sendable, Equatable, Codable {
    case ms30
    case ms20
    case ms10

    public static var defaultValue: ContinuousTraffic = .ms20

    public var uiValue: String {
        switch self {
        case .ms30:
            "0.7"
        case .ms20:
            "1"
        case .ms10:
            "2"
        }
    }

    public var actualValue: Int {
        switch self {
        case .ms30:
            30
        case .ms20:
            20
        case .ms10:
            10
        }
    }

    public init(actualValue: UInt32?) {
        guard let actualValue
        else {
            self = Self.defaultValue
            return
        }

        switch actualValue {
        case 30:
            self = .ms30
        case 20:
            self = .ms20
        case 10:
            self = .ms10
        default:
            self = Self.defaultValue
        }
    }

    public static func fromIndex(_ index: Double) -> ContinuousTraffic {
        let safeIndex = Int(index.rounded())
        return ContinuousTraffic(rawValue: safeIndex) ?? .ms30
    }
}
