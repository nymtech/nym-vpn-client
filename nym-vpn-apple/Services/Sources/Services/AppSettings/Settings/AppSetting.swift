import SwiftUI
import Theme

public struct AppSetting {
    public enum Appearance: Int, CaseIterable {
        #if os(iOS)
        case automatic
        #endif
        case light
        case dark

        public var colorScheme: ColorScheme? {
            switch self {
            case .light:
                return .light
            case .dark:
                return .dark
            #if os(iOS)
            case .automatic:
                return nil
            #endif
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
    }
}
