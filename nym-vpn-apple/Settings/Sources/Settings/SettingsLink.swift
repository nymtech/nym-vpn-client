import Foundation

enum SettingsLink: Hashable, Identifiable {
    case addCredentials
    case theme
    case feedback
    case support
    case legal
    case survey
    case surveySuccess

    var id: String {
        String(describing: self)
    }
}
