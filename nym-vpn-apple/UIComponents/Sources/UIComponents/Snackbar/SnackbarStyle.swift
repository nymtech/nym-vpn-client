import SwiftUI
import Theme

public enum SnackbarStyle {
    case info

    var backgroundColor: Color {
        switch self {
        case .info:
            NymColor.sysOnSecondary
        }
    }

    var textColor: Color {
        switch self {
        case .info:
            NymColor.sysOnSurface
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
            NymColor.sysOnSurface
        }
    }
}
