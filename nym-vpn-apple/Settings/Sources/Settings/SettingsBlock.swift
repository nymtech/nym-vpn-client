import Theme

/// A searchable item that SettingsView draws itself instead of as a list row.
enum SettingsBlock: Hashable {
    case getStarted
    case appVersion(String)

    var title: String {
        switch self {
        case .getStarted:
            "home.getStarted".localizedString
        case let .appVersion(title):
            title
        }
    }

    var isAboveRows: Bool {
        self == .getStarted
    }
}
