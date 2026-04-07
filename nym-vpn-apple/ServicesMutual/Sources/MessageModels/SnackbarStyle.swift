import SwiftUI
import Theme

public enum SnackbarStyle {
    case info
    case noIcon
    case cta
    case expiry
    case passphrase

    public var backgroundColor: Color {
        switch self {
        case .info, .noIcon, .cta, .expiry, .passphrase:
            NymColor.elevation
        }
    }

    public var textColor: Color {
        switch self {
        case .info, .noIcon, .cta, .expiry, .passphrase:
            NymColor.primary
        }
    }

    public var systemIconName: String? {
        switch self {
        case .info:
            "info.circle"
        case .noIcon, .cta, .expiry, .passphrase:
            nil
        }
    }

    public var iconColor: Color {
        switch self {
        case .info, .noIcon, .cta, .expiry, .passphrase:
            NymColor.primary
        }
    }

    public var showsCloseButton: Bool {
        switch self {
        case .passphrase:
            false
        case .info, .noIcon, .cta, .expiry:
            true
        }
    }
}
