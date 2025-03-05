import Combine
import SwiftUI
import AppSettings
import HelperManager
import Theme

@MainActor public final class HelperInstallViewModel: ObservableObject {
    private let appSettings: AppSettings
    private let helperManager: HelperManager
    private let afterInstallAction: HelperAfterInstallAction

    private var daemonStateCancellable: AnyCancellable?
    private var timerCancellable: AnyCancellable?
    private var lastDaemonState = DaemonState.unknown

    let navTitle = "helper.installView.pageTitle".localizedString
    let infoText = "helper.installView.daemonText".localizedString

    @Binding var path: NavigationPath
    @Published var steps: [HelperInstallStep] = []
    @Published var secondsRemaining: Int = 5
    @Published var error: Error?

    public init(
        path: Binding<NavigationPath>,
        afterInstallAction: HelperAfterInstallAction,
        appSettings: AppSettings = .shared,
        helperManager: HelperManager = .shared
    ) {
        _path = path
        self.afterInstallAction = afterInstallAction
        self.appSettings = appSettings
        self.helperManager = helperManager

        setup()
    }
}

@MainActor extension HelperInstallViewModel {
    func buttonAction() {
        error = nil
        switch helperManager.daemonState {
        case .requiresManualRemoval:
            break
        case .unknown, .registered, .requiresAuthorization:
            install()
        case .authorized, .updating:
            break
        case .running:
            navigateBack()
        case .requiresUpdate:
            update()
        }
    }

    func buttonTitle() -> String {
        switch helperManager.daemonState {
        case .unknown:
            "helper.installView.registerAndAuhtorize".localizedString
        case .registered, .requiresAuthorization:
            "helper.installView.authorize".localizedString
        case .authorized:
            "helper.installView.waitingToStart".localizedString
        case .running:
            "\("helper.installView.continue".localizedString) \(secondsRemaining)..."
        case .requiresUpdate:
            "helper.installView.update".localizedString
        case .updating:
            "helper.installView.updating".localizedString
        case .requiresManualRemoval:
            "helper.installView.verifying".localizedString
        }
    }

    func buttonColor() -> Color {
        switch helperManager.daemonState {
        case .unknown, .registered, .requiresAuthorization, .running, .requiresUpdate:
            NymColor.accent
        case .authorized, .updating, .requiresManualRemoval:
            NymColor.sysSecondary
        }
    }

    func copyCommands() {
        let text = """
sudo launchctl unload /Library/LaunchDaemons/net.nymtech.vpn.helper.plist
sudo rm /Library/LaunchDaemons/net.nymtech.vpn.helper.plist
sudo rm /Library/PrivilegedHelperTools/net.nymtech.vpn.helper
sfltool resetbtm
"""
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(text, forType: .string)
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
}

// MARK: - Private -
@MainActor private extension HelperInstallViewModel {
    func setup() {
        setupDaemonStateObserver()
        updateSteps()
    }

    func setupDaemonStateObserver() {
        daemonStateCancellable = helperManager.$daemonState
            .receive(on: DispatchQueue.main)
            .removeDuplicates()
            .delay(for: .seconds(3), scheduler: DispatchQueue.main)
            .sink { [weak self] newState in
                guard let self, newState != lastDaemonState else { return }
                lastDaemonState = newState
                updateSteps()
                startCountdownIfNeeded()
            }
    }

    func updateSteps() {
        var newSteps = [HelperInstallStep]()
        if requiresDaemonMigration() {
            newSteps.append(.uninstallOldDeamon)
        } else {
            newSteps.append(contentsOf: [
                .register(isRegistered: isDaemonRegistered()),
                .authorize(isAuthorized: isDaemonAuthorized()),
                .running(isRunning: isDaemonRunning()),
                .versionCheck(requiresUpdate: requiresUpdate())
            ])
        }
        steps = newSteps
    }

    func install() {
        do {
            try helperManager.install()
        } catch {
            self.error = error
        }
    }

    func update() {
        do {
            try helperManager.update()
        } catch {
            self.error = error
        }
    }

    func startCountdownIfNeeded() {
        guard isDaemonRegistered(),
              isDaemonAuthorized(),
              isDaemonRunning(),
              !requiresUpdate(),
              !requiresDaemonMigration(),
              timerCancellable == nil
        else {
            return
        }
        timerCancellable = Timer.publish(every: 1.0, on: .main, in: .common)
            .autoconnect()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                guard let self = self else { return }
                if let daemonStateCancellable {
                    daemonStateCancellable.cancel()
                    self.daemonStateCancellable = nil
                }
                secondsRemaining -= 1

                if secondsRemaining <= 0 {
                    timerCancellable?.cancel()
                    completeAction()
                }
            }
    }
}

@MainActor private extension HelperInstallViewModel {
    func requiresDaemonMigration() -> Bool {
        helperManager.daemonState == .requiresManualRemoval
    }

    func isDaemonRegistered() -> Bool {
        switch helperManager.daemonState {
        case .registered, .requiresAuthorization, .authorized, .running, .requiresUpdate, .updating:
            true
        default:
            false
        }
    }

    func isDaemonAuthorized() -> Bool {
        switch helperManager.daemonState {
        case .authorized, .running, .requiresUpdate, .updating:
            true
        default:
            false
        }
    }

    func isDaemonRunning() -> Bool {
        switch helperManager.daemonState {
        case .running, .requiresUpdate:
            true
        default:
            false
        }
    }

    func requiresUpdate() -> Bool {
        guard isDaemonRunning() else { return true }
        switch helperManager.daemonState {
        case .requiresUpdate:
            return true
        default:
            return false
        }
    }
}
