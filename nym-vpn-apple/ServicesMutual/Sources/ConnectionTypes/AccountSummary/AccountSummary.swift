import Foundation

public struct AccountSummary {
    public var validUntilDate: Date?
    public var trafficUsedGb: Int?
    public var trafficLimitGb: Int?
    public var trafficResetDate: Date?
    public var accountAddress: String
    public var canonicalAccountAddress: String?
    public var accountAuthMethod: [AccountAuthMethod]
    public var isLinked: Bool
    public var isActive: Bool

    public init(
        validUntilDate: Date?,
        trafficUsedGb: Int?,
        trafficLimitGb: Int?,
        trafficResetDate: Date?,
        accountAddress: String,
        cannonicalAccountAddress: String?,
        accountAuthMethod: [AccountAuthMethod],
        isLinked: Bool,
        isActive: Bool
    ) {
        self.validUntilDate = validUntilDate
        self.trafficUsedGb = trafficUsedGb
        self.trafficLimitGb = trafficLimitGb
        self.trafficResetDate = trafficResetDate
        self.accountAddress = accountAddress
        self.canonicalAccountAddress = cannonicalAccountAddress
        self.accountAuthMethod = accountAuthMethod
        self.isLinked = isLinked
        self.isActive = isActive
    }

    public init(
        validUntilTimeInterval: Int64?,
        trafficUsedGb: UInt64?,
        trafficLimitGb: UInt64?,
        trafficResetTimeInterval: Int64?,
        accountAddress: String,
        cannonicalAccountAddress: String?,
        accountAuthMethod: [AccountAuthMethod],
        isLinked: Bool,
        isActive: Bool
    ) {
        self.validUntilDate = validUntilTimeInterval.map { Date(timeIntervalSince1970: TimeInterval($0)) }
        self.trafficUsedGb = trafficUsedGb.flatMap(Int.init(exactly:))
        self.trafficLimitGb = trafficLimitGb.flatMap(Int.init(exactly:))
        self.trafficResetDate = trafficResetTimeInterval.map { Date(timeIntervalSince1970: TimeInterval($0)) }
        self.accountAddress = accountAddress
        self.canonicalAccountAddress = cannonicalAccountAddress
        self.accountAuthMethod = accountAuthMethod
        self.isLinked = isLinked
        self.isActive = isActive
    }
}
