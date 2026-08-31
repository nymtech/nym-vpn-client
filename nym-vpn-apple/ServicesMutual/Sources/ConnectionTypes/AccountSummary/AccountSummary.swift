import Foundation
import SwiftUI
import Theme

public struct AccountSummary: Codable {
    public var validUntilDate: Date?
    public var trafficUsedGb: Int?
    public var trafficLimitGb: Int?
    public var trafficResetDate: Date?
    public var accountAddress: String
    public var canonicalAccountAddress: String?
    public var accountAuthMethod: [AccountAuthMethod]
    public var isLinked: Bool
    public var isActive: Bool
    public var isAutoRenewEnabled: Bool
    public var subscription: Subscription?
    public var dataUnavailable: Bool

    public init(
        validUntilDate: Date?,
        trafficUsedGb: Int?,
        trafficLimitGb: Int?,
        trafficResetDate: Date?,
        accountAddress: String,
        cannonicalAccountAddress: String?,
        accountAuthMethod: [AccountAuthMethod],
        isLinked: Bool,
        isActive: Bool,
        isAutoRenewEnabled: Bool,
        subscription: Subscription?,
        dataUnavailable: Bool = false
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
        self.isAutoRenewEnabled = isAutoRenewEnabled
        self.subscription = subscription
        self.dataUnavailable = dataUnavailable
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
        isActive: Bool,
        isAutoRenewEnabled: Bool,
        subscription: Subscription?,
        dataUnavailable: Bool = false
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
        self.isAutoRenewEnabled = isAutoRenewEnabled
        self.subscription = subscription
        self.dataUnavailable = dataUnavailable
    }

    public var formattedValidUntilDate: String? {
        guard isActive, let validUntilDate else { return nil }
        let formatter = DateFormatter()
        formatter.locale = .autoupdatingCurrent
        formatter.dateStyle = .long
        formatter.timeStyle = .none
        return formatter.string(from: validUntilDate)
    }

    public var planValidUntilAttributedString: AttributedString? {
        guard subscription?.status != .pending else { return nil }
        if !isActive {
            var result = AttributedString("noActivePlan".localizedString)
            result.foregroundColor = NymColor.error
            return result
        }
        guard let formattedDate = formattedValidUntilDate else { return nil }
        if isExpiringSoon || isExpiringWarning {
            var result = AttributedString("\("planExpiresOn".localizedString) \(formattedDate)")
            result.foregroundColor = statusColor
            return result
        }
        var result = AttributedString("\("planValidUntil".localizedString) \(formattedDate)")
        result.foregroundColor = statusColor
        return result
    }

    public var isExpiringSoon: Bool {
        guard let validUntilDate else { return false }
        let daysRemaining = Calendar.current.dateComponents([.day], from: Date(), to: validUntilDate).day ?? 0
        let subscriptionKind = subscription?.subscription.kind
        let isShortPlan = subscriptionKind == .oneMonth || subscriptionKind == .freepass
        let threshold = isShortPlan ? 2 : 15
        return daysRemaining < threshold
    }

    public var isExpiringWarning: Bool {
        guard let validUntilDate else { return false }
        let daysRemaining = Calendar.current.dateComponents([.day], from: Date(), to: validUntilDate).day ?? 0
        let subscriptionKind = subscription?.subscription.kind
        let isShortPlan = subscriptionKind == .oneMonth || subscriptionKind == .freepass
        let threshold = isShortPlan ? 7 : 60
        return daysRemaining < threshold
    }

    public var statusColor: Color {
        if isExpiringSoon {
            return NymColor.orange
        } else if isExpiringWarning {
            return NymColor.warning
        }
        return NymColor.accent
    }
}
