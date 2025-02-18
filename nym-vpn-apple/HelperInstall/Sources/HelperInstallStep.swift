import SwiftUI
import Theme

enum HelperInstallStep: Hashable, Identifiable {
    case uninstallOldDeamon
    case register(isRegistered: Bool)
    case authorize(isAuthorized: Bool)
    case running(isRunning: Bool)
    case versionCheck(requiresUpdate: Bool, requiredVersion: String, currentVersion: String)

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
        case let .versionCheck(needsUpdate, requiredVersion, currentVersion):
            if currentVersion != "unknown", requiredVersion != currentVersion {
                 """
                \("helper.installView.step.versionMismatch".localizedString)
                \("helper.installView.step.requiredVersion".localizedString) - \(requiredVersion)
                \("helper.installView.step.currentVersion".localizedString) - \(currentVersion)
                """
            } else if needsUpdate {
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
        case let .versionCheck(requiresUpdate, _, _):
            requiresUpdate ? "xmark.seal.fill" : "checkmark.seal.fill"
        }
    }

    var imageColor: Color {
        switch self {
        case let .register(isOn), let .authorize(isOn), let .running(isOn):
            isOn ? NymColor.primaryOrange : NymColor.noInternet
        case let .versionCheck(_, requiredVersion, currentVersion):
            requiredVersion == currentVersion ? NymColor.primaryOrange : NymColor.noInternet
        case .uninstallOldDeamon:
            NymColor.noInternet
        }
    }
}

extension HelperInstallStep: Equatable {
    static func == (lhs: HelperInstallStep, rhs: HelperInstallStep) -> Bool {
        switch (lhs, rhs) {
        case let (.register(isOnLhs), .register(isOnRhs)),
            let (.authorize(isOnLhs), .authorize(isOnRhs)),
            let (.running(isOnLhs), .running(isOnRhs)):
            isOnLhs == isOnRhs
        case let (.versionCheck(requiresUpdateLhs, requiredVersionLhs, currentVersionLhs),
            .versionCheck(requiresUpdateRhs, requiredVersionRhs, currentVersionRhs)):
            requiresUpdateLhs == requiresUpdateRhs
            && requiredVersionLhs == requiredVersionRhs
            && currentVersionLhs == currentVersionRhs
        default:
            false
        }
    }
}
