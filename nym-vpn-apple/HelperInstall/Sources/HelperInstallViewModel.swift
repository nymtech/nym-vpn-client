import Combine
import SwiftUI
import AppSettings
import Constants
import GRPCManager
import HelperManager
import Theme
import UIComponents

@MainActor public final class HelperInstallViewModel: ObservableObject {
    private let appSettings: AppSettings
    private let helperManager: HelperManager
    private let afterInstallAction: HelperAfterInstallAction
    private let grpcManager: GRPCManager

    private var cancellables = Set<AnyCancellable>()

    let navTitle = "helper.installView.pageTitle".localizedString
    let infoText = "helper.installView.daemonText".localizedString
    let copiedSuccesfullyMessage = "helper.installView.copyToClipboardSuccess".localizedString

    @Binding var path: NavigationPath
    @Published var secondsRemaining: Int = 5
    @Published var isSnackBarDisplayed = false
    @Published var isSuccessModalDisplayed = false
    @Published var isMigrationModalDisplayed = false

    var updateAvailableOverlayConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            systemIconImageName: "checkmark",
            systemIconImageColor: NymColor.action,
            titleLocalizedString: "helper.installView.successModal.title".localizedString,
            subtitleLocalizedString: "helper.installView.successModal.subtitle".localizedString,
            yesLocalizedString: "helper.installview.backToMainScreen".localizedString,
            yesAction: { [weak self] in
                self?.navigateBack()
            }
        )
    }

    var migrationOverlayConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            systemIconImageColor: NymColor.error,
            titleLocalizedString: "helper.installView.daemonMigrationRequired".localizedString,
            subtitleLocalizedString: "helper.installView.step.uninstallOldDaemon".localizedString,
            yesLocalizedString: "helper.installView.copy".localizedString,
            yesAction: { [weak self] in
                self?.copyCommands()
            },
            shouldCloseAfterYesAction: false
        )
    }

    func thirdStepAttributedString() -> AttributedString? {
        try? AttributedString(markdown: "\("helper.installView.thirdStep".localizedString) [\("helper.installView.thirdStep.supportTeam".localizedString)](\(Constants.supportURL.rawValue))")
    }

    public init(
        path: Binding<NavigationPath>,
        afterInstallAction: HelperAfterInstallAction,
        appSettings: AppSettings = .shared,
        helperManager: HelperManager = .shared,
        grpcManager: GRPCManager = .shared
    ) {
        _path = path
        self.afterInstallAction = afterInstallAction
        self.appSettings = appSettings
        self.helperManager = helperManager
        self.grpcManager = grpcManager

        setup()
    }
}

@MainActor extension HelperInstallViewModel {
    func copyCommands() {
        let text = """
sudo launchctl unload /Library/LaunchDaemons/net.nymtech.vpn.helper.plist
sudo rm /Library/LaunchDaemons/net.nymtech.vpn.helper.plist
sudo rm /Library/PrivilegedHelperTools/net.nymtech.vpn.helper
sfltool resetbtm
"""
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(text, forType: .string)

        isSnackBarDisplayed = true
    }
}

// MARK: - Navigation -
@MainActor extension HelperInstallViewModel {
    func completeAction() {
        navigateBack()
        afterInstallAction.completion?()
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func openSystemSettings() {
        helperManager.openSystemSettings()
    }
}

// MARK: - Private -
@MainActor private extension HelperInstallViewModel {
    func setup() {
        setupIsServingObserver()
        setupHelperStateObserver()
    }

    func setupIsServingObserver() {
        grpcManager.$isServing
            .receive(on: DispatchQueue.main)
            .removeDuplicates()
            .delay(for: .seconds(3), scheduler: DispatchQueue.main)
            .sink { [weak self] isServing in
                guard let self, isServing else { return }
                isSuccessModalDisplayed = true
            }
            .store(in: &cancellables)
    }

    func setupHelperStateObserver() {
        helperManager.$daemonState
            .receive(on: DispatchQueue.main)
            .sink { [weak self] daemonState in
                self?.isMigrationModalDisplayed = daemonState == .requiresManualRemoval
            }
            .store(in: &cancellables)
    }
}
