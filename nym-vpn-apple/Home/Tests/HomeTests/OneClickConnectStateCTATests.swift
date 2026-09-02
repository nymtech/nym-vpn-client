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
}
