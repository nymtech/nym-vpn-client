import ConnectionTypes

extension AccountSummary {
    func shouldShowRenewButton(isAutoRenew: Bool) -> Bool {
        let isPending = subscription?.status == .pending
        return !isPending && !isAutoRenew && (isExpiringSoon || !isActive)
    }

    var renewButtonTitle: String {
        isActive ? "settings.account.renewNow".localizedString : "purchasePlan.chooseMyPlan".localizedString
    }
}
