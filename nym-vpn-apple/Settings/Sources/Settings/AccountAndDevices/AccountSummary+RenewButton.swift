import ConnectionTypes

extension AccountSummary {
    func shouldShowRenewButton(isAutoRenew: Bool) -> Bool {
        !isAutoRenew && (isExpiringSoon || !isActive)
    }

    var renewButtonTitle: String {
        isActive ? "settings.account.renewNow".localizedString : "purchasePlan.chooseMyPlan".localizedString
    }
}
