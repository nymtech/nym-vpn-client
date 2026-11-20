import XCTest
import UIComponents
@testable import Settings

final class SettingsSearchTests: XCTestCase {
    private func row(_ title: String) -> SettingsListItemViewModel {
        SettingsListItemViewModel(accessory: .arrow, title: title, action: {})
    }

    private var sections: [SettingsSection<AppSettingsSectionKind>] {
        [
            SettingsSection(kind: .protection, viewModels: [row("Kill switch"), row("Block ads")]),
            SettingsSection(kind: .connection, viewModels: [row("Anti-censorship"), row("Customize DNS")]),
            SettingsSection(kind: .app, viewModels: [row("Apparence"), row("Écran")])
        ]
    }

    func testEmptyQueryKeepsEverySection() {
        XCTAssertEqual(SettingsSearch.filter(sections, query: "  ").map(\.kind), [.protection, .connection, .app])
    }

    func testMatchStaysUnderItsSection() {
        let result = SettingsSearch.filter(sections, query: "dns")
        XCTAssertEqual(result.map(\.kind), [.connection])
        XCTAssertEqual(result.first?.viewModels.map(\.title), ["Customize DNS"])
    }

    func testMatchIgnoresCaseAndAccents() {
        let result = SettingsSearch.filter(sections, query: "ECRAN")
        XCTAssertEqual(result.first?.viewModels.map(\.title), ["Écran"])
    }

    func testNoMatchReturnsNothing() {
        XCTAssertEqual(SettingsSearch.filter(sections, query: "zzz").count, 0)
    }
}
