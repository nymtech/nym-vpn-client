import SwiftUI

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
                print("light")
                return .light
            case .dark:
                print("dark")
                return .dark
            #if os(iOS)
            case .automatic:
                print("system")
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
}
