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
        switch helperManager.daemonState {
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
        }
    }

    func buttonColor() -> Color {
        switch helperManager.daemonState {
        case .unknown, .registered, .requiresAuthorization, .running, .requiresUpdate:
            NymColor.primaryOrange
        case .authorized, .updating:
            NymColor.sysSecondary
        }
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
            .receive(on: RunLoop.main)
            .sink { [weak self] newState in
                guard let self, newState != lastDaemonState else { return }
                lastDaemonState = newState
                updateSteps()
                startCountdownIfNeeded()
            }
    }

    func updateSteps() {
        steps = [
            .register(isRegistered: isDaemonRegistered()),
            .authorize(isAuthorized: isDaemonAuthorized()),
            .running(isRunning: isDaemonRunning()),
            .versionCheck(
                requiresUpdate: requiresUpdate(),
                requiredVersion: helperManager.requiredVersion,
                currentVersion: helperManager.currentVersion
            )
        ]
    }

    func install() {
        try? helperManager.install()
    }

    func update() {
        try? helperManager.update()
    }

    func startCountdownIfNeeded() {
        guard isDaemonRegistered(), isDaemonAuthorized(), isDaemonRunning(), !requiresUpdate() else { return }
        timerCancellable = Timer.publish(every: 1.0, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                guard let self = self else { return }
                if let daemonStateCancellable {
                    daemonStateCancellable.cancel()
                    self.daemonStateCancellable = nil
                }
                self.secondsRemaining -= 1

                if self.secondsRemaining <= 0 {
                    self.timerCancellable?.cancel()
                    self.completeAction()
                }
            }
    }
}

@MainActor private extension HelperInstallViewModel {
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
