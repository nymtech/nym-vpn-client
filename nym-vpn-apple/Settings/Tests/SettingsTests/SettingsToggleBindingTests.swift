import XCTest
import AppSettings
import ConnectionManager
@testable import Settings

@MainActor
final class SettingsToggleBindingTests: XCTestCase {
    func testSetAdBlockingUpdatesAppSettings() {
        let settings = AppSettings.shared
        let prior = settings.isAdBlockerEnabled
        defer { settings.isAdBlockerEnabled = prior }

        ConnectionManager.shared.setAdBlocking(true)
        XCTAssertTrue(settings.isAdBlockerEnabled)

        ConnectionManager.shared.setAdBlocking(false)
        XCTAssertFalse(settings.isAdBlockerEnabled)
    }

    func testSetLanBypassUpdatesAppSettings() {
        let settings = AppSettings.shared
        let prior = settings.isLanBypassEnabled
        defer { settings.isLanBypassEnabled = prior }

        ConnectionManager.shared.setLanBypassEnabled(true)
        XCTAssertTrue(settings.isLanBypassEnabled)

        ConnectionManager.shared.setLanBypassEnabled(false)
        XCTAssertFalse(settings.isLanBypassEnabled)
    }
}
