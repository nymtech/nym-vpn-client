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

        let killSwitchCountBefore = viewModel.sections.first { $0.kind == .killSwitch }?.viewModels.count
        XCTAssertNotNil(killSwitchCountBefore)

        AppSettings.shared.isCredentialImported = true
        CredentialsManager.shared.accountSummary = AccountSummary.makeFake(
            daysRemaining: 30,
            kind: .oneMonth,
            isAutoRenew: true,
            baseAddress: "test"
        )
        viewModel.updateAccountSectionOnly()

        let killSwitchCountAfter = viewModel.sections.first { $0.kind == .killSwitch }?.viewModels.count
        XCTAssertEqual(killSwitchCountBefore, killSwitchCountAfter)
        XCTAssertNotNil(viewModel.sections.first { $0.kind == .account })
    }

    private func waitForSections(_ viewModel: SettingsViewModel) async {
        for _ in 0..<50 where viewModel.sections.isEmpty {
            try? await Task.sleep(for: .milliseconds(20))
        }
        XCTAssertFalse(viewModel.sections.isEmpty)
    }
}
