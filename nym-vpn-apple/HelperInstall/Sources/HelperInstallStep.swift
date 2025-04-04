import SwiftUI
import Theme

enum HelperInstallStep: Hashable, Identifiable {
    case uninstallOldDeamon
    case register(isRegistered: Bool)
    case authorize(isAuthorized: Bool)
    case running(isRunning: Bool)
    case versionCheck(requiresUpdate: Bool)

    var id: Self {
        self
    }

    var title: String {
        switch self {
        case .uninstallOldDeamon:
            "helper.installView.step.uninstallOldDaemon".localizedString
        case .register:
            "helper.installView.step.install".localizedString
        case .authorize:
            "helper.installView.step.authorize".localizedString
        case let .running(isRunning):
            isRunning
            ? "helper.installView.step.isRunning".localizedString
            : "helper.installView.step.waitingToStart".localizedString
        case let .versionCheck(requiresUpdate):
            if requiresUpdate {
                "helper.installView.step.verifyingVersion".localizedString
            } else {
                "helper.installView.step.versionMatch".localizedString
            }
        }
    }

    var systemImageName: String {
        switch self {
        case .uninstallOldDeamon:
            "trash.circle"
        case let .register(isRegistered):
            isRegistered ? "shield.lefthalf.filled.badge.checkmark" : "xmark.shield"
        case let .authorize(isAuthorized):
            isAuthorized ? "checkmark.shield" : "xmark.shield"
        case let .running(isRunning):
            isRunning ? "bolt.badge.checkmark" : "bolt.badge.xmark"
        case let .versionCheck(requiresUpdate):
            requiresUpdate ? "xmark.seal.fill" : "checkmark.seal.fill"
        }
    }

    var imageColor: Color {
        switch self {
        case let .register(isOn), let .authorize(isOn), let .running(isOn):
            isOn ? NymColor.accent : NymColor.error
        case let .versionCheck(requiresUpdate):
            requiresUpdate ? NymColor.error : NymColor.accent
        case .uninstallOldDeamon:
            NymColor.error
        }
    }
}

extension HelperInstallStep: Equatable {
    static func == (lhs: HelperInstallStep, rhs: HelperInstallStep) -> Bool {
        switch (lhs, rhs) {
        case let (.register(isOnLhs), .register(isOnRhs)),
            let (.authorize(isOnLhs), .authorize(isOnRhs)),
            let (.running(isOnLhs), .running(isOnRhs)),
            let (.versionCheck(isOnLhs), .versionCheck(isOnRhs)):
            isOnLhs == isOnRhs
        default:
            false
        }
    }
}
