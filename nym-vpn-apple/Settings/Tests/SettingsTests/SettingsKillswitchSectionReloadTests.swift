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

    func testSettingsLayoutFollowsReviewedOrder() async {
        let wasSignedIn = AppSettings.shared.isCredentialImported
        defer { AppSettings.shared.isCredentialImported = wasSignedIn }
        AppSettings.shared.isCredentialImported = false
        let viewModel = makeViewModel()
        await waitForSections(viewModel)

        func titles(_ kind: AppSettingsSectionKind) -> [String]? {
            viewModel.sections.first { $0.kind == kind }?.viewModels.map(\.title)
        }
        XCTAssertEqual(titles(.account), [], "signed out: the Account section holds only the Get started button")
        XCTAssertEqual(
            titles(.protection),
            ["settings.adblock.title", "settings.killswitch.title", "settings.mixnetTuning.title"].map(\.localizedString)
        )
        XCTAssertEqual(
            titles(.connection).map { Array($0.suffix(3)) },
            ["settings.censorship.title", "settings.profiles.title", "settings.dns.title"].map(\.localizedString)
        )
        let allTitles = viewModel.sections.flatMap { $0.viewModels.map(\.title) }
        XCTAssertFalse(allTitles.contains("settings.systemStatus".localizedString))
        XCTAssertFalse(allTitles.contains("settings.redeemCode".localizedString))
        XCTAssertFalse(allTitles.contains("settings.passphrase".localizedString))
    }

    func testSignedInShowsOneYourAccountRow() async {
        let wasSignedIn = AppSettings.shared.isCredentialImported
        defer { AppSettings.shared.isCredentialImported = wasSignedIn }
        AppSettings.shared.isCredentialImported = true
        let viewModel = makeViewModel()
        await waitForSections(viewModel)

        let account = viewModel.sections.filter { $0.kind == .account }
        XCTAssertEqual(account.count, 1)
        XCTAssertEqual(account.first?.viewModels.map(\.title), ["settings.yourAccount".localizedString])
    }

    private func waitForSections(_ viewModel: SettingsViewModel) async {
        for _ in 0..<50 where viewModel.sections.isEmpty {
            try? await Task.sleep(for: .milliseconds(20))
        }
        XCTAssertFalse(viewModel.sections.isEmpty)
    }
}
