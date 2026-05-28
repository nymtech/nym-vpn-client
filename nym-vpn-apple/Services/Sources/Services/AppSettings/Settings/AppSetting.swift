import SwiftUI
import Theme

#if os(iOS)
public enum AppIcon: String, CaseIterable, Identifiable {
    case `default` = "default"
    case dark = "dark"
    case calculator = "calculator"
    case notes = "notes"

    public var id: String { rawValue }

    /// The alternate icon name passed to `UIApplication.setAlternateIconName(_:)`.
    /// `nil` means the primary (default) icon.
    public var alternateName: String? {
        switch self {
        case .default: return nil
        case .dark: return "AppIcon-Dark"
        case .calculator: return "AppIcon-Calculator"
        case .notes: return "AppIcon-Notes"
        }
    }

    /// Asset name used to preview the icon in the settings grid.
    public var previewAssetName: String {
        switch self {
        case .default: return "AppIcon"
        case .dark: return "AppIcon-Dark"
        case .calculator: return "AppIcon-Calculator"
        case .notes: return "AppIcon-Notes"
        }
    }

    /// Localization key for the display title.
    public var localizedTitleKey: String {
        switch self {
        case .default: return "settings.appIcon.default"
        case .dark: return "settings.appIcon.dark"
        case .calculator: return "settings.appIcon.calculator"
        case .notes: return "settings.appIcon.notes"
        }
    }
}
#endif

public struct AppSetting {
    public enum Appearance: Int, CaseIterable {
        case automatic
        case light
        case dark

        public var colorScheme: ColorScheme? {
            switch self {
            case .light:
                return .light
            case .dark:
                return .dark
            case .automatic:
                return nil
            }
        }
#if os(iOS)
        public var userInterfaceStyle: UIUserInterfaceStyle {
            switch self {
            case .automatic:
                return .unspecified
            case .light:
                return .light
            case .dark:
                return .dark
            }
        }
#elseif os(macOS)
        public var nsAppearance: NSAppearance? {
            switch self {
            case .automatic:
                return nil
            case .light:
                return NSAppearance(named: .aqua)
            case .dark:
                return NSAppearance(named: .darkAqua)
            }
        }
#endif
    }

    public enum AppMode: Int, CaseIterable {
        case both
        case menubarOnly
        case dockOnly

        public var localizedTitle: String {
            switch self {
            case .menubarOnly:
                "settings.appMode.menuBarOnly".localizedString
            case .dockOnly:
                "settings.appMode.dockOnly".localizedString
            case .both:
                "settings.appMode.both".localizedString
            }
        }

#if os(macOS)
        public var activationPolicy: NSApplication.ActivationPolicy {
            switch self {
            case .menubarOnly:
                    .accessory
            case .dockOnly, .both:
                    .regular
            }
        }
#endif
    }
}
