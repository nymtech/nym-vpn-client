import SwiftUI
import MessageModels
import Theme

extension View {
    public func snackbar(
        isDisplayed: Binding<Bool>,
        message: SnackBarMessage?
    ) -> some View {
        self.modifier(
            SnackbarModifier(isDisplayed: isDisplayed, message: message)
        )
    }

    public func nymText(color: Color, style: NymTextStyle) -> some View {
        self
            .foregroundStyle(color)
            .textStyle(style)
    }
}

struct SnackbarModifier: ViewModifier {
    @Binding var isDisplayed: Bool
    var message: SnackBarMessage?

    func body(content: Content) -> some View {
        ZStack {
            content
            SnackbarView(isDisplayed: $isDisplayed, message: message, appSettings: .shared)
        }
    }
}
