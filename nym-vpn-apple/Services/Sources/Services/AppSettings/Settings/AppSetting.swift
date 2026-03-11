import SwiftUI
import Theme

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
