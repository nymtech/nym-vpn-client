import SwiftUI
import Theme

enum HelperInstallStep: Hashable, Identifiable {
    case register(isRegistered: Bool)
    case authorize(isAuthorized: Bool)
    case running(isRunning: Bool)

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
        }
    }

    var imageColor: Color {
        switch self {
        case let .register(isOn), let .authorize(isOn), let .running(isOn):
            isOn ? NymColor.primaryOrange : NymColor.noInternet
        }
    }
}
