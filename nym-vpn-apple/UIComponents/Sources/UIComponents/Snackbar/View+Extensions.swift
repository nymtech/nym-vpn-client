import SwiftUI
import MessageModels

extension View {
    public func snackbar(
        isDisplayed: Binding<Bool>,
        message: SnackBarMessage?
    ) -> some View {
        self.modifier(
            SnackbarModifier(isDisplayed: isDisplayed, message: message)
        )
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
