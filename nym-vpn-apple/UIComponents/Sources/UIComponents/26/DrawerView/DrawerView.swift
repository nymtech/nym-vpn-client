import SwiftUI
import Theme

private enum DrawerViewConstants {
    static let slideOutOffset: CGFloat = 800
}

/// A bottom-anchored, floating card drawer that animates its content when `tag` changes.
///
/// Place `DrawerView` inside a full-height container (e.g., a `ZStack`). A `Spacer` pushes
/// the card to the bottom edge, where it floats above the screen with rounded corners and
/// standard padding. When `tag` changes, the card slides completely off the bottom, calls
/// `onTransitionCompleted` (use this to update the rendered content), then springs back up.
public struct DrawerView<Tag: Hashable, Content: View>: View {
    let tag: Tag
    let onTransitionCompleted: () -> Void
    @ViewBuilder let content: () -> Content

    @State private var offsetY: CGFloat = 0

    public init(
        tag: Tag,
        onTransitionCompleted: @escaping () -> Void = {},
        @ViewBuilder content: @escaping () -> Content
    ) {
        self.tag = tag
        self.onTransitionCompleted = onTransitionCompleted
        self.content = content
    }

    public var body: some View {
        content()
            .frame(maxWidth: .infinity)
            .background(Color.Nym.surfaceElev)
            .clipShape(RoundedRectangle(cornerRadius: NymSpacing.section))
            .frame(maxWidth: NymSpacing.drawerMaxWidth)
            .padding(.horizontal, NymSpacing.standard)
            .offset(y: offsetY)
            .onChange(of: tag) { _, _ in
                slideOut()
            }
    }

    private func slideOut() {
        withAnimation(.easeIn) {
            offsetY = DrawerViewConstants.slideOutOffset
        } completion: {
            onTransitionCompleted()
            withAnimation(.spring) {
                offsetY = 0
            }
        }
    }
}
