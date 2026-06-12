import XCTest
@testable import Settings

final class AppIconTests: XCTestCase {
    func testAlternateNameRoundTrips() {
        for icon in AppIcon.allCases {
            XCTAssertEqual(AppIcon(alternateName: icon.alternateName), icon, "\(icon) failed alternateName round-trip")
        }
    }

    func testPrimaryHasNilAlternateName() {
        XCTAssertNil(AppIcon.primary.alternateName)
    }

    func testUnknownAlternateNameFallsBackToPrimary() {
        XCTAssertEqual(AppIcon(alternateName: "garbage"), .primary)
    }

    func testEveryCaseHasNonEmptyPreviewName() {
        for icon in AppIcon.allCases {
            XCTAssertFalse(icon.previewImageName.isEmpty)
        }
    }

    func testEveryCaseHasNonEmptyTitle() {
        for icon in AppIcon.allCases {
            XCTAssertFalse(icon.title.isEmpty)
        }
    }
}

@MainActor
final class FakeAppIconChanger: AppIconChanging {
    var currentAlternateIconName: String?
    private(set) var setCalls: [String?] = []
    var errorToThrow: Error?

    init(current: String? = nil) {
        self.currentAlternateIconName = current
    }

    func setAlternateIconName(_ name: String?) async throws {
        setCalls.append(name)
        if let errorToThrow { throw errorToThrow }
        currentAlternateIconName = name
    }
}

@MainActor
final class AppIconViewModelTests: XCTestCase {
    func testInitReflectsCurrentAlternateName() {
        let vm = AppIconViewModel(changer: FakeAppIconChanger(current: "AppIcon-Notes"))
        XCTAssertEqual(vm.current, .notes)
    }

    func testSelectSuccessUpdatesCurrentAndCallsChanger() async {
        let changer = FakeAppIconChanger(current: nil)
        let vm = AppIconViewModel(changer: changer)

        await vm.select(.calculator)

        XCTAssertEqual(vm.current, .calculator)
        XCTAssertEqual(changer.setCalls, ["AppIcon-Calculator"])
    }

    func testSelectSameIconIsNoOp() async {
        let changer = FakeAppIconChanger(current: nil)
        let vm = AppIconViewModel(changer: changer)

        await vm.select(.primary)

        XCTAssertEqual(changer.setCalls, [])
    }

    func testSelectFailureLeavesCurrentUnchanged() async {
        let changer = FakeAppIconChanger(current: nil)
        changer.errorToThrow = NSError(domain: "test", code: 1)
        let vm = AppIconViewModel(changer: changer)

        await vm.select(.notes)

        XCTAssertEqual(vm.current, .primary)
        XCTAssertEqual(changer.setCalls, ["AppIcon-Notes"])
    }
}
