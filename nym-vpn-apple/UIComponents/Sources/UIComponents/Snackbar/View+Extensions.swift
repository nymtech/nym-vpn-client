import SwiftUI

extension View {
    public func snackbar(
        isDisplayed: Binding<Bool>,
        style: SnackbarStyle,
        message: String
    ) -> some View {
        self.modifier(
            SnackbarModifier(
                isDisplayed: isDisplayed,
                style: style,
                message: message
            )
        )
    }
}

struct SnackbarModifier: ViewModifier {
    @Binding var isDisplayed: Bool
    var style: SnackbarStyle
    var message: String

    func body(content: Content) -> some View {
        ZStack {
            content
            SnackbarView(
                isDisplayed: $isDisplayed,
                style: style,
                message: message
            )
        }
    }
}
