import Foundation

public extension AccountSummary {
    /// Account-status renew row: shown when active-but-not-auto-renewing, or inactive.
    /// Mirrors `AccountAndDevicesView+AccountStatus.accountStatusSection`.
    var shouldShowRenewRow: Bool { !isActive || !isAutoRenewEnabled }

    /// "Account on nym.com" link row: shown while the account is not yet linked.
    var shouldShowLinkAccountRow: Bool { !isLinked }
}
