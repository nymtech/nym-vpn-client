import SwiftUI
import Theme

public enum SnackbarStyle {
    case info
    case noIcon

    public var backgroundColor: Color {
        switch self {
        case .info, .noIcon:
            NymColor.gray1
        }
    }

    public var textColor: Color {
        switch self {
        case .info, .noIcon:
            NymColor.primary
        }
    }

    public var systemIconName: String? {
        switch self {
        case .info:
            "info.circle"
        case .noIcon:
            nil
        }
    }

    public var iconColor: Color {
        switch self {
        case .info, .noIcon:
            NymColor.primary
        }
    }
}
