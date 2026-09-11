import Foundation
import Testing
import AccountPrefetchGates

struct DisconnectedHomeCTATests {
    @Test func missingCredentialIsGetStarted() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: false,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false,
                hasAccountSummary: false
            ) == .getStarted
        )
    }

    @Test func inactiveSummaryIsChoosePlan() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false,
                hasAccountSummary: true
            ) == .choosePlan
        )
    }

    @Test func fetchFailedWithoutActiveAccountIsUnreachable() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: true,
                accountSummaryLastFetchFailed: true,
                isAccountActive: false,
                hasAccountSummary: false
            ) == .accountUnreachable
        )
    }

    @Test func transientFetchFailedDoesNotOfferPlan() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: true,
                accountSummaryLastFetchFailed: true,
                isAccountActive: false,
                hasAccountSummary: true
            ) == .accountUnreachable
        )
    }

    @Test func importedActiveIsConnect() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: true,
                hasAccountSummary: true
            ) == .connect
        )
    }

    @Test func importedWithoutSummaryIsUnknownNotConnect() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false,
                hasAccountSummary: false
            ) == .checking
        )
    }

    @Test func knownInactiveWithoutSummaryIsChoosePlanNotChecking() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false,
                hasAccountSummary: false,
                isAccountKnownInactive: true
            ) == .choosePlan
        )
    }

    @Test func fetchFailedStillWinsOverKnownInactive() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: true,
                accountSummaryLastFetchFailed: true,
                isAccountActive: false,
                hasAccountSummary: false,
                isAccountKnownInactive: true
            ) == .accountUnreachable
        )
    }
}
