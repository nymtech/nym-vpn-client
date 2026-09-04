import XCTest
@testable import Home
import AccountPrefetchGates

final class OneClickConnectStateCTATests: XCTestCase {
    func testDisconnectedCTAMapsToNamedConnectStates() {
        XCTAssertEqual(OneClickConnectState.disconnected(.getStarted), .noAccount)
        XCTAssertEqual(OneClickConnectState.disconnected(.choosePlan), .noSubscription)
        XCTAssertEqual(OneClickConnectState.disconnected(.accountUnreachable), .accountUnreachable)
        XCTAssertEqual(OneClickConnectState.disconnected(.checking), .checkingAccount)
        XCTAssertEqual(OneClickConnectState.disconnected(.connect), .disconnected)
    }

    func testImportedInactiveWithoutSummaryIsCheckingAccount() {
        let cta = DisconnectedHomeCTA.resolve(
            isCredentialImported: true,
            accountSummaryLastFetchFailed: false,
            isAccountActive: false,
            hasAccountSummary: false
        )
        XCTAssertEqual(cta, .checking)
        XCTAssertEqual(OneClickConnectState.disconnected(cta), .checkingAccount)
    }

    func testKnownInactiveWithoutSummaryIsChoosePlanAndNoSubscription() {
        let cta = DisconnectedHomeCTA.resolve(
            isCredentialImported: true,
            accountSummaryLastFetchFailed: false,
            isAccountActive: false,
            hasAccountSummary: false,
            isAccountKnownInactive: true
        )
        XCTAssertEqual(cta, .choosePlan)
        XCTAssertEqual(OneClickConnectState.disconnected(cta), .noSubscription)
    }

    func testImportedInactiveWithSummaryIsChoosePlanAndNoSubscription() {
        let cta = DisconnectedHomeCTA.resolve(
            isCredentialImported: true,
            accountSummaryLastFetchFailed: false,
            isAccountActive: false,
            hasAccountSummary: true
        )
        XCTAssertEqual(cta, .choosePlan)
        XCTAssertEqual(OneClickConnectState.disconnected(cta), .noSubscription)
    }
}
