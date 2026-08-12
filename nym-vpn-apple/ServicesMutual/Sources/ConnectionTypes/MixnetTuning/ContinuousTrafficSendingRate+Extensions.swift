import NymVPNLib

extension ContinuousTrafficSendingRate: @retroactive Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let index = try container.decode(Int.self)
        switch index {
        case 0:
            self = .ms30
        case 1:
            self = MixnetTrafficDefaults().defaultContinuousTraffic()
        case 2:
            self = .ms10
        default:
            self = MixnetTrafficDefaults().defaultContinuousTraffic()
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .ms30:
            try container.encode(0)
        case .ms20:
            try container.encode(1)
        case .ms10:
            try container.encode(2)
        }
    }
}

extension ContinuousTrafficSendingRate {
    public init(fromValue value: UInt32?) {
        guard let value else {
            self = MixnetTrafficDefaults().defaultContinuousTraffic()
            return
        }
        switch value {
        case 30:
            self = .ms30
        case 20:
            self = MixnetTrafficDefaults().defaultContinuousTraffic()
        case 10:
            self = .ms10
        default:
            self = MixnetTrafficDefaults().defaultContinuousTraffic()
        }
    }

    public var uiThroughput: String {
        switch self {
        case .ms30:
            "0.7"
        case .ms20:
            "1"
        case .ms10:
            "2"
        }
    }
}
