import Theme

public enum BackgroundTraffic: Int, CaseIterable, Sendable, Equatable, Codable {
    case ms200
    case ms40
    case ms20
    case ms10

    public static var defaultValue: BackgroundTraffic = .ms200

    public var uiValue: String {
        switch self {
        case .ms200:
            "\("mixnetTuning.base".localizedString)\n"
        case .ms40:
            "\("mixnetTuning.balanced".localizedString)\n5x"
        case .ms20:
            "\("mixnetTuning.medium".localizedString)\n10x"
        case .ms10:
            "\("mixnetTuning.high".localizedString)\n20x"
        }
    }

    public var actualValue: Int {
        switch self {
        case .ms200:
            200
        case .ms40:
            40
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
        case 200:
            self = .ms200
        case 40:
            self = .ms40
        case 20:
            self = .ms20
        case 10:
            self = .ms10
        default:
            self = Self.defaultValue
        }
    }

    public static func fromIndex(_ index: Double) -> BackgroundTraffic {
        let safeIndex = Int(index.rounded())
        return BackgroundTraffic(rawValue: safeIndex) ?? .ms200
    }
}
