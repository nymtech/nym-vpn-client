import Foundation

public enum AccountReportPlatform: String, CaseIterable {
    case iOS
    case macOS
}

public enum AccountActionKind: String {
    case renewPlan
    case manageSubscriptionExternal
    case manageSubscriptionInApp
    case logout

    /// Human-readable description of what tapping the button does (for the report).
    public var reportDescription: String {
        switch self {
        case .renewPlan:
            return "Renew → plan purchase (macOS: autologin renew; iOS: passphrase → purchase)"
        case .manageSubscriptionExternal:
            return "Manage subscription → autologin to web account"
        case .manageSubscriptionInApp:
            return "Manage subscription → iOS native subscriptions sheet"
        case .logout:
            return "Logout → disconnect, then remove credential"
        }
    }
}

public struct AccountButton: Equatable {
    public let titleKey: String
    public let kind: AccountActionKind
    public let isDestructive: Bool

    public init(titleKey: String, kind: AccountActionKind, isDestructive: Bool) {
        self.titleKey = titleKey
        self.kind = kind
        self.isDestructive = isDestructive
    }
}

/// The ordered set of buttons the Account & Devices view renders for a given
/// account state. Mirrors `AccountAndDevicesView` render order:
/// renew (in account-status card) → manage-subscription (web) → iOS in-app manage → logout.
public func accountButtons(
    for summary: AccountSummary?,
    platform: AccountReportPlatform,
    isTestFlight: Bool
) -> [AccountButton] {
    guard let summary else { return [] }
    var buttons: [AccountButton] = []

    if summary.shouldShowRenewRow {
        buttons.append(AccountButton(titleKey: "settings.account.renewNow", kind: .renewPlan, isDestructive: false))
    }
    buttons.append(AccountButton(titleKey: "settings.account.manageSubscription", kind: .manageSubscriptionExternal, isDestructive: false))
    if platform == .iOS && !isTestFlight {
        buttons.append(AccountButton(titleKey: "settings.manageSubscription", kind: .manageSubscriptionInApp, isDestructive: false))
    }
    buttons.append(AccountButton(titleKey: "settings.logout", kind: .logout, isDestructive: true))
    return buttons
}
