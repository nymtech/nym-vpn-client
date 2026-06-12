import XCTest
import OnboardingGates

final class AccountSetupRepairGateTests: XCTestCase {
    func testNoRepairWhenSetupNotNeeded() {
        XCTAssertEqual(
            AccountSetupRepairGate.repairAction(needsSetup: false, hasAccountToken: false),
            .none
        )
        XCTAssertEqual(
            AccountSetupRepairGate.repairAction(needsSetup: false, hasAccountToken: true),
            .none
        )
    }

    func testRepairRegistersAccountWhenTokenMissing() {
        XCTAssertEqual(
            AccountSetupRepairGate.repairAction(needsSetup: true, hasAccountToken: false),
            .registerAccount
        )
    }

    func testRepairPreparesRegisteredAccountWhenTokenPresent() {
        XCTAssertEqual(
            AccountSetupRepairGate.repairAction(needsSetup: true, hasAccountToken: true),
            .prepareRegisteredAccount
        )
    }
}
