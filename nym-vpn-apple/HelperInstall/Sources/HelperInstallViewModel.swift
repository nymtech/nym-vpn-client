import Combine
import SwiftUI
import AppSettings
import HelperManager
import Theme

@MainActor public final class HelperInstallViewModel: ObservableObject {
    private let appSettings: AppSettings
    private let helperManager: HelperManager
    private let afterInstallAction: HelperAfterInstallAction

    private var cancellables = Set<AnyCancellable>()
    private var timerCancellable: AnyCancellable?

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
        case .authorized:
            break
        case .running:
            navigateBack()
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
        }
    }

    func buttonColor() -> Color {
        switch helperManager.daemonState {
        case .unknown, .registered, .requiresAuthorization, .running:
            NymColor.primaryOrange
        case .authorized:
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
        helperManager.$daemonState
            .receive(on: RunLoop.main)
            .sink { [weak self] _ in
                self?.updateSteps()
                self?.startCountdownIfNeeded()
            }
            .store(in: &cancellables)
    }

    func updateSteps() {
        steps = [
            .register(isRegistered: isDaemonRegistered()),
            .authorize(isAuthorized: isDaemonAuthorized()),
            .running(isRunning: isDaemonRunning())
        ]
    }

    func install() {
        try? helperManager.install()
    }

    func startCountdownIfNeeded() {
        guard isDaemonRegistered(), isDaemonAuthorized(), isDaemonRunning() else { return }
        timerCancellable = Timer.publish(every: 1.0, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                guard let self = self else { return }

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
        case .registered, .requiresAuthorization, .authorized, .running:
            true
        default:
            false
        }
    }

    func isDaemonAuthorized() -> Bool {
        switch helperManager.daemonState {
        case .authorized, .running:
            true
        default:
            false
        }
    }

    func isDaemonRunning() -> Bool {
        switch helperManager.daemonState {
        case .running:
            true
        default:
            false
        }
    }
}
