import SwiftUI
import Theme

public enum SnackbarStyle {
    case info
    case noIcon
    case cta

    public var backgroundColor: Color {
        switch self {
        case .info, .noIcon, .cta:
            NymColor.elevation
        }
    }

    public var textColor: Color {
        switch self {
        case .info, .noIcon, .cta:
            NymColor.primary
        }
    }

    public var systemIconName: String? {
        switch self {
        case .info:
            "info.circle"
        case .noIcon, .cta:
            nil
        }
    }

    public var iconColor: Color {
        switch self {
        case .info, .noIcon, .cta:
            NymColor.primary
        }
    }
}
