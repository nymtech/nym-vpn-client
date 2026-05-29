import SwiftUI
import Theme

public struct ModalOverlayView<Content: View>: View {
    private let dismissOnOverlayTap: Bool
    private let horizontalPadding: CGFloat
    private let maxWidth: CGFloat
    private let content: Content

    @Binding private var isDisplayed: Bool

    public init(
        isDisplayed: Binding<Bool>,
        dismissOnOverlayTap: Bool = true,
        horizontalPadding: CGFloat = 40,
        maxWidth: CGFloat = MagicNumbers.moreMaxWidth,
        @ViewBuilder content: () -> Content
    ) {
        _isDisplayed = isDisplayed
        self.dismissOnOverlayTap = dismissOnOverlayTap
        self.horizontalPadding = horizontalPadding
        self.maxWidth = maxWidth
        self.content = content()
    }

    public var body: some View {
        ZStack {
            Rectangle()
                .foregroundColor(.black)
                .opacity(0.3)
                .background(Color.clear)
                .contentShape(Rectangle())
                .onTapGesture {
                    guard dismissOnOverlayTap else { return }
                    withAnimation(.easeInOut) {
                        isDisplayed = false
                    }
                }

            HStack {
                Spacer()
                    .frame(width: horizontalPadding)

                content
                    .background(Color.Nym.surface)
                    .cornerRadius(16)

                Spacer()
                    .frame(width: horizontalPadding)
            }
            .frame(maxWidth: maxWidth)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .edgesIgnoringSafeArea(.all)
    }
}
