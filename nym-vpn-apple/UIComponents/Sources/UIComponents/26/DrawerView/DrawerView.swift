import SwiftUI
import Theme

/// A bottom-anchored, floating card drawer.
///
/// Visual only: wraps `content` in the standard surface card with rounded corners,
/// max width, and horizontal padding. Slide animation is owned by the caller so
/// adjacent views (e.g., a segmented control above the drawer) can share the
/// same offset and move together.
public struct DrawerView<Content: View>: View {
    @ViewBuilder let content: () -> Content

    public init(@ViewBuilder content: @escaping () -> Content) {
        self.content = content
    }

    public var body: some View {
        content()
            .frame(maxWidth: .infinity)
            .background(Color.Nym.surface)
            .clipShape(RoundedRectangle(cornerRadius: NymSpacing.section))
            .frame(maxWidth: NymSpacing.drawerMaxWidth)
            .padding(.horizontal, NymSpacing.standard)
    }
}

public enum DrawerSlide {
    public static let offset: CGFloat = 800
}
