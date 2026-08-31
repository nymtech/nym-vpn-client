import Testing
@testable import UIComponents
import TunnelStatus

struct ConnectButtonStateCTATests {
    @Test func missingCredentialIsNoAccount() {
        #expect(
            ConnectButtonState(
                tunnelStatus: .disconnected,
                isCredentialImported: false
            ) == .noAccount
        )
    }

    @Test func inactiveSummaryIsNoSubscription() {
        #expect(
            ConnectButtonState(
                tunnelStatus: .disconnected,
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false,
                hasAccountSummary: true
            ) == .noSubscription
        )
    }

    @Test func fetchFailedIsAccountUnreachableNotConnect() {
        #expect(
            ConnectButtonState(
                tunnelStatus: .disconnected,
                isCredentialImported: true,
                accountSummaryLastFetchFailed: true,
                isAccountActive: false,
                hasAccountSummary: false
            ) == .accountUnreachable
        )
    }

    @Test func importedInactiveWithoutSummaryIsNoSubscription() {
        #expect(
            ConnectButtonState(
                tunnelStatus: .disconnected,
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false,
                hasAccountSummary: false
            ) == .noSubscription
        )
    }

    @Test func activeAccountIsConnect() {
        #expect(
            ConnectButtonState(
                tunnelStatus: .disconnected,
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: true,
                hasAccountSummary: true
            ) == .connect
        )
    }
}
