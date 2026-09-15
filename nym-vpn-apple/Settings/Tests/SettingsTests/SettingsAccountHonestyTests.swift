import Testing
import ConnectionTypes
import Theme
@testable import Settings

@MainActor
struct SettingsAccountHonestyTests {
    @Test func nilSummaryFetchFailedIsUnreachableNotChoosePlan() {
        #expect(
            SettingsViewModel.nilSummaryAccountCopy(
                lastFetchFailed: true,
                isRegistrationInFlight: false
            ) == .unreachable
        )
    }

    @Test func nilSummaryInFlightIsRequestingZkNyms() {
        #expect(
            SettingsViewModel.nilSummaryAccountCopy(
                lastFetchFailed: false,
                isRegistrationInFlight: true
            ) == .requestingZkNyms
        )
        #expect(
            SettingsViewModel.nilSummaryAccountCopy(
                lastFetchFailed: true,
                isRegistrationInFlight: true
            ) == .requestingZkNyms
        )
    }

    @Test func nilSummaryIdleIsCheckingNotChoosePlan() {
        #expect(
            SettingsViewModel.nilSummaryAccountCopy(
                lastFetchFailed: false,
                isRegistrationInFlight: false
            ) == .checking
        )
    }

    @Test func inactiveRenewButtonTitleIsChoosePlanNotRenewNow() {
        let summary = AccountSummary.makeFake(
            daysRemaining: nil,
            kind: .oneYear,
            isAutoRenew: false,
            baseAddress: "a"
        )
        #expect(summary.renewButtonTitle == "purchasePlan.chooseMyPlan".localizedString)
        #expect(summary.renewButtonTitle != "settings.account.renewNow".localizedString)
    }

    @Test func activeRenewButtonTitleIsRenewNow() {
        let summary = AccountSummary.makeFake(
            daysRemaining: 7,
            kind: .oneMonth,
            isAutoRenew: true,
            baseAddress: "a"
        )
        #expect(summary.renewButtonTitle == "settings.account.renewNow".localizedString)
    }
}
