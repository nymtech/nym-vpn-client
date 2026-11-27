import XCTest
import Theme
import UIComponents
@testable import Settings

final class SettingsSearchTests: XCTestCase {
    private func row(_ title: String, _ subtitle: String? = nil) -> SettingsListItemViewModel {
        SettingsListItemViewModel(accessory: .arrow, title: title, subtitle: subtitle, action: {})
    }

    private var sections: [SettingsSection<AppSettingsSectionKind>] {
        [
            SettingsSection(kind: .account, viewModels: [row("Redeem a code")]),
            SettingsSection(kind: .protection, viewModels: [row("Block ads", "Blocking ads and trackers"), row("Kill switch")]),
            SettingsSection(kind: .connection, viewModels: [row("Anti-censorship"), row("Customize DNS")]),
            SettingsSection(kind: .app, viewModels: [row("Apparence"), row("Écran")]),
            SettingsSection(kind: .footer, viewModels: [row("Log out")])
        ]
    }

    private func blocks(_ kind: AppSettingsSectionKind) -> [SettingsBlock] {
        switch kind {
        case .account:
            [.getStarted]
        case .footer:
            [.appVersion("Version 2.2.2")]
        default:
            []
        }
    }

    private func filter(_ query: String) -> [SettingsSearchSection] {
        SettingsSearch.filter(sections, query: query, blocks: blocks)
    }

    private func titles(_ result: [SettingsSearchSection]) -> [[String]] {
        result.map { $0.blocks.map(\.title) + $0.rows.map(\.title) }
    }

    func testBlankQueryKeepsEverySectionInOrder() {
        let result = filter("  ")
        XCTAssertEqual(result.map(\.kind), [.account, .protection, .connection, .app, .footer])
        XCTAssertEqual(result.first?.blocks, [.getStarted])
    }

    func testRowTitleMatchStaysUnderItsSection() {
        let result = filter(" dns ")
        XCTAssertEqual(result.map(\.kind), [.connection])
        XCTAssertEqual(titles(result), [["Customize DNS"]])
    }

    func testMatchIgnoresCaseAndAccents() {
        XCTAssertEqual(titles(filter("ECRAN")), [["Écran"]])
    }

    func testRowSubtitleMatchKeepsThatRow() {
        XCTAssertEqual(titles(filter("trackers")), [["Block ads"]])
    }

    func testSectionTitleMatchKeepsTheWholeSection() {
        let result = filter("settings.section.protection".localizedString)
        XCTAssertEqual(result.map(\.kind), [.protection])
        XCTAssertEqual(titles(result), [["Block ads", "Kill switch"]])
    }

    func testBlocksAreFoundByTheirTitles() {
        XCTAssertEqual(titles(filter(SettingsBlock.getStarted.title)), [[SettingsBlock.getStarted.title]])
        XCTAssertEqual(titles(filter("version")), [["Version 2.2.2"]])
        XCTAssertEqual(titles(filter("log out")), [["Log out"]])
    }

    func testNoMatchReturnsNothing() {
        XCTAssertTrue(filter("zzz").isEmpty)
    }
}
