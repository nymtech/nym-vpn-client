import SwiftUI
import Theme

enum HelperInstallStep: Hashable, Identifiable {
    case register(isRegistered: Bool)
    case authorize(isAuthorized: Bool)
    case running(isRunning: Bool)
    case versionCheck(requiresUpdate: Bool, requiredVersion: String, currentVersion: String)

    var id: Self {
        self
    }

    var title: String {
        switch self {
        case .register:
            "helper.installView.step.install".localizedString
        case .authorize:
            "helper.installView.step.authorize".localizedString
        case let .running(isRunning):
            isRunning
            ? "helper.installView.step.isRunning".localizedString
            : "helper.installView.step.waitingToStart".localizedString
        case let .versionCheck(needsUpdate, requiredVersion, currentVersion):
            if needsUpdate, currentVersion != "unknown" {
                """
                \("helper.installView.step.versionMismatch".localizedString)
                \("helper.installView.step.requiredVersion".localizedString) - \(requiredVersion)
                \("helper.installView.step.currentVersion".localizedString) - \(currentVersion)
                """
            } else {
                currentVersion == requiredVersion
                ? "helper.installView.step.versionMatch".localizedString
                : "helper.installView.step.verifyingVersion".localizedString
            }
        }
    }

    var systemImageName: String {
        switch self {
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
        case let .versionCheck(requiresUpdate, _, _):
            requiresUpdate ? NymColor.noInternet : NymColor.primaryOrange
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
