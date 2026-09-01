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
                isAccountActive: false
            ) == .noSubscription
        )
    }

    @Test func fetchFailedIsAccountUnreachableNotConnect() {
        #expect(
            ConnectButtonState(
                tunnelStatus: .disconnected,
                isCredentialImported: true,
                accountSummaryLastFetchFailed: true,
                isAccountActive: false
            ) == .accountUnreachable
        )
    }

    @Test func importedInactiveWithoutSummaryIsNoSubscription() {
        #expect(
            ConnectButtonState(
                tunnelStatus: .disconnected,
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false
            ) == .noSubscription
        )
    }

    @Test func activeAccountIsConnect() {
        #expect(
            ConnectButtonState(
                tunnelStatus: .disconnected,
                isCredentialImported: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: true
            ) == .connect
        )
    }

#if os(macOS)
    @Test func choosePlanMenuBarItemIsLabelOnly() {
        #expect(!ConnectButtonState.noSubscription.menuBarItemIsAction)
        #expect(!ConnectButtonState.accountUnreachable.menuBarItemIsAction)
        #expect(ConnectButtonState.connect.menuBarItemIsAction)
    }
#endif
}
