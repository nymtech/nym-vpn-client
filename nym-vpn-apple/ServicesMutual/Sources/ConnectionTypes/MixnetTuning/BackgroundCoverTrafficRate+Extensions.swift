import NymVPNLib

extension BackgroundCoverTrafficRate: @retroactive Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let index = try container.decode(Int.self)
        switch index {
        case 0:
            self = MixnetTrafficDefaults().defaultBackgroundTraffic()
        case 1:
            self = .ms40
        case 2:
            self = .ms20
        case 3:
            self = .ms10
        default:
            self = MixnetTrafficDefaults().defaultBackgroundTraffic()
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .ms200:
            try container.encode(0)
        case .ms40:
            try container.encode(1)
        case .ms20:
            try container.encode(2)
        case .ms10:
            try container.encode(3)
        }
    }
}

extension BackgroundCoverTrafficRate {
    public init(fromValue value: UInt32?) {
        guard let value else {
            self = MixnetTrafficDefaults().defaultBackgroundTraffic()
            return
        }
        switch value {
        case 200:
            self = MixnetTrafficDefaults().defaultBackgroundTraffic()
        case 40:
            self = .ms40
        case 20:
            self = .ms20
        case 10:
            self = .ms10
        default:
            self = MixnetTrafficDefaults().defaultBackgroundTraffic()
        }
    }
}
