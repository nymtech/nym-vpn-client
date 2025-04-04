import SwiftUI
import Theme

public enum SnackbarStyle {
    case info

    var backgroundColor: Color {
        switch self {
        case .info:
            NymColor.gray1
        }
    }

    var textColor: Color {
        switch self {
        case .info:
            NymColor.primary
        }
    }

    var systemIconName: String? {
        switch self {
        case .info:
            "info.circle"
        }
    }

    var iconColor: Color {
        switch self {
        case .info:
            NymColor.primary
        }
    }
}
