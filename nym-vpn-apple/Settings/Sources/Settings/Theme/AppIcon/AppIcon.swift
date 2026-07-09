import Theme

enum AppIcon: CaseIterable, Equatable, Hashable {
    case primary
    case calculator
    case notes

    var alternateName: String? {
        switch self {
        case .primary:
            return nil
        case .calculator: 
            return "AppIcon-Calculator"
        case .notes: 
            return "AppIcon-Notes"
        }
    }

    /// Imageset name (Settings `.module` bundle) for the picker thumbnail.
    var previewImageName: String {
        switch self {
        case .primary:
            return "iconPreviewDefault"
        case .calculator: 
            return "iconPreviewCalculator"
        case .notes: 
            return "iconPreviewNotes"
        }
    }

    var title: String {
        switch self {
        case .primary:
            return "appIcon.default".localizedString
        case .calculator:
            return "appIcon.calculator".localizedString
        case .notes:
            return "appIcon.notes".localizedString
        }
    }

    init(alternateName: String?) {
        self = AppIcon.allCases.first { $0.alternateName == alternateName } ?? .primary
    }
}
