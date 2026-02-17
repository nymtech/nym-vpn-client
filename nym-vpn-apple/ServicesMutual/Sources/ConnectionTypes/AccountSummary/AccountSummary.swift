import Foundation

public struct AccountSummary {
    public var validUntilDate: Date?
    public var trafficUsedGb: Int?
    public var trafficLimitGb: Int?
    public var trafficResetDate: Date?

    public init(validUntilDate: Date?, trafficUsedGb: Int?, trafficLimitGb: Int?, trafficResetDate: Date?) {
        self.validUntilDate = validUntilDate
        self.trafficUsedGb = trafficUsedGb
        self.trafficLimitGb = trafficLimitGb
        self.trafficResetDate = trafficResetDate
    }

    public init(
        validUntilTimeInterval: Int64?,
        trafficUsedGb: UInt64?,
        trafficLimitGb: UInt64?,
        trafficResetTimeInterval: Int64?
    ) {
        self.validUntilDate = validUntilTimeInterval.map { Date(timeIntervalSince1970: TimeInterval($0)) }
        self.trafficUsedGb = trafficUsedGb.flatMap(Int.init(exactly:))
        self.trafficLimitGb = trafficLimitGb.flatMap(Int.init(exactly:))
        self.trafficResetDate = trafficResetTimeInterval.map { Date(timeIntervalSince1970: TimeInterval($0)) }
    }
}
