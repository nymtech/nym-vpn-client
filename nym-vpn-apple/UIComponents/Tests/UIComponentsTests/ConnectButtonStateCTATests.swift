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

    @Test func importedWithoutSummaryIsCheckingAccount() {
        #expect(
            ConnectButtonState(
                tunnelStatus: .disconnected,
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false,
                hasAccountSummary: false
            ) == .checkingAccount
        )
    }

    @Test func knownInactiveWithoutSummaryIsNoSubscription() {
        #expect(
            ConnectButtonState(
                tunnelStatus: .disconnected,
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false,
                hasAccountSummary: false,
                isAccountKnownInactive: true
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

#if os(macOS)
    @Test func choosePlanMenuBarItemIsLabelOnly() {
        #expect(!ConnectButtonState.noSubscription.menuBarItemIsAction)
        #expect(!ConnectButtonState.accountUnreachable.menuBarItemIsAction)
        #expect(!ConnectButtonState.checkingAccount.menuBarItemIsAction)
        #expect(ConnectButtonState.connect.menuBarItemIsAction)
    }
#endif
}
