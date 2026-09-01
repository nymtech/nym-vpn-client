import Foundation
import Testing
import AccountPrefetchGates

struct DisconnectedHomeCTATests {
    @Test func missingCredentialIsGetStarted() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: false,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false
            ) == .getStarted
        )
    }

    @Test func inactiveSummaryIsChoosePlan() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false
            ) == .choosePlan
        )
    }

    @Test func fetchFailedWithoutActiveAccountIsUnreachable() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: true,
                accountSummaryLastFetchFailed: true,
                isAccountActive: false
            ) == .accountUnreachable
        )
    }

    @Test func transientFetchFailedDoesNotOfferPlan() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: true,
                accountSummaryLastFetchFailed: true,
                isAccountActive: false
            ) == .accountUnreachable
        )
    }

    @Test func importedActiveIsConnect() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: true
            ) == .connect
        )
    }

    @Test func importedWithoutSummaryIsChoosePlanNotConnect() {
        #expect(
            DisconnectedHomeCTA.resolve(
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false
            ) == .choosePlan
        )
    }
}
