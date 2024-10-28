import SwiftUI
import AppSettings

public struct AppearanceUpdate: ViewModifier {
    @EnvironmentObject private var appSettings: AppSettings

    public func body(content: Content) -> some View {
        content
#if os(macOS)
            .preferredColorScheme(appSettings.currentAppearance.colorScheme)
#endif
    }
}

public extension View {
    func appearanceUpdate() -> some View {
        modifier(AppearanceUpdate())
    }
}
