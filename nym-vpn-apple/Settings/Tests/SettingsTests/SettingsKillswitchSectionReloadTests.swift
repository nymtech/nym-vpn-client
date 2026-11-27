import SwiftUI
import XCTest
import AppSettings
import ConfigurationManager
import ConnectionManager
import ConnectionTypes
import CredentialsManager
import ExternalLinkManager
import FeatureFlagsManager
import ImpactGenerator
#if os(iOS)
import PurchasesManager
#endif
import Theme
@testable import Settings

@MainActor
final class SettingsKillswitchSectionReloadTests: XCTestCase {
    private func makeViewModel() -> SettingsViewModel {
#if os(macOS)
        SettingsViewModel(
            isServing: .constant(true),
            path: .constant(NavigationPath()),
            appSettings: .shared,
            configurationManager: .shared,
            connectionManager: .shared,
            credentialsManager: .shared,
            externalLinkManager: .shared,
            featureFlagsManager: .shared,
            impactGenerator: .shared
        )
#else
        SettingsViewModel(
            path: .constant(NavigationPath()),
            appSettings: .shared,
            configurationManager: .shared,
            connectionManager: .shared,
            credentialsManager: .shared,
            externalLinkManager: .shared,
            featureFlagsManager: .shared,
            impactGenerator: .shared,
            purchasesManager: .shared
        )
#endif
    }

    func testAccountSectionUpdatePreservesKillSwitchSection() async {
        let viewModel = makeViewModel()
        await waitForSections(viewModel)

        let killSwitchCountBefore = viewModel.sections.first { $0.kind == .protection }?.viewModels.count
        XCTAssertNotNil(killSwitchCountBefore)

        AppSettings.shared.isCredentialImported = true
        CredentialsManager.shared.accountSummary = AccountSummary.makeFake(
            daysRemaining: 30,
            kind: .oneMonth,
            isAutoRenew: true,
            baseAddress: "test"
        )
        viewModel.updateAccountSectionOnly()

        let killSwitchCountAfter = viewModel.sections.first { $0.kind == .protection }?.viewModels.count
        XCTAssertEqual(killSwitchCountBefore, killSwitchCountAfter)
        XCTAssertNotNil(viewModel.sections.first { $0.kind == .account })
    }

    func testSignedOutLayout() async {
        let wasSignedIn = AppSettings.shared.isCredentialImported
        defer { AppSettings.shared.isCredentialImported = wasSignedIn }
        AppSettings.shared.isCredentialImported = false
        let viewModel = makeViewModel()
        await waitForSections(viewModel)

        func titles(_ kind: AppSettingsSectionKind) -> [String]? {
            viewModel.sections.first { $0.kind == kind }?.viewModels.map(\.title)
        }
        XCTAssertEqual(titles(.account), ["settings.redeemCode".localizedString])
        XCTAssertEqual(viewModel.blocks(for: .account), [.getStarted])
        XCTAssertEqual(
            titles(.protection),
            ["settings.adblock.title", "settings.killswitch.title", "settings.mixnetTuning.title"].map(\.localizedString)
        )
        XCTAssertEqual(
            titles(.connection).map { Array($0.suffix(3)) },
            ["settings.censorship.title", "settings.profiles.title", "settings.dns.title"].map(\.localizedString)
        )
        XCTAssertEqual(titles(.footer), [])
        XCTAssertEqual(viewModel.blocks(for: .footer), [.appVersion(viewModel.versionTitle)])
        XCTAssertEqual(viewModel.sections.last?.kind, .footer)
    }

    func testSignedInLayout() async {
        let wasSignedIn = AppSettings.shared.isCredentialImported
        defer { AppSettings.shared.isCredentialImported = wasSignedIn }
        AppSettings.shared.isCredentialImported = true
        let viewModel = makeViewModel()
        await waitForSections(viewModel)

        var accountTitles = ["settings.yourAccount".localizedString]
#if os(iOS)
        accountTitles.append("settings.passphrase".localizedString)
#endif
        accountTitles.append("settings.redeemCode".localizedString)
        let account = viewModel.sections.filter { $0.kind == .account }
        XCTAssertEqual(account.count, 1)
        XCTAssertEqual(account.first?.viewModels.map(\.title), accountTitles)
        XCTAssertEqual(viewModel.blocks(for: .account), [])
        let footer = viewModel.sections.first { $0.kind == .footer }
        XCTAssertEqual(footer?.viewModels.map(\.title), ["settings.logout".localizedString])
    }

    private func waitForSections(_ viewModel: SettingsViewModel) async {
        for _ in 0..<50 where viewModel.sections.isEmpty {
            try? await Task.sleep(for: .milliseconds(20))
        }
        XCTAssertFalse(viewModel.sections.isEmpty)
    }
}
