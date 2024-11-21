import SwiftUI
import AppSettings

public struct SnackbarView: View {
    private let appSettings: AppSettings
    private let style: SnackbarStyle
    private let message: String

    @Binding private var isDisplayed: Bool

    public init(
        isDisplayed: Binding<Bool>,
        style: SnackbarStyle,
        message: String,
        appSettings: AppSettings = AppSettings.shared
    ) {
        self._isDisplayed = isDisplayed
        self.style = style
        self.message = message
        self.appSettings = appSettings
    }

    public var body: some View {
        VStack {
            if isDisplayed {
                HStack(alignment: .center, spacing: 12) {
                    messageStyleImage()
                    messageText()
                    Spacer()
                    closeButton()
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 16)
                .frame(maxWidth: .infinity, minHeight: 35)
                .padding(.vertical, 8)
                .background(style.backgroundColor)
                .cornerRadius(10)
                .padding(.horizontal, 16)
                .padding(.top, appSettings.isSmallScreen ? 64 : 80) // CustomNavBarSize + 16
                .transition(isDisplayed ? .move(edge: .trailing) : .move(edge: .leading))
                .animation(.easeInOut, value: isDisplayed)
            }
            Spacer()
        }
    }
}

extension SnackbarView {
    @ViewBuilder
    func messageStyleImage() -> some View {
        if let name = style.systemIconName {
            Image(systemName: name)
                .resizable()
                .foregroundStyle(style.iconColor)
                .aspectRatio(contentMode: .fit)
                .frame(width: 14, height: 14)
        }
    }

    @ViewBuilder
    func messageText() -> some View {
        Text(message)
            .foregroundColor(style.textColor)
            .font(.system(size: 14))
            .frame(alignment: .leading)
    }

    @ViewBuilder
    func closeButton() -> some View {
        Image(systemName: "xmark")
            .resizable()
            .foregroundStyle(style.iconColor)
            .aspectRatio(contentMode: .fit)
            .frame(width: 14, height: 14)
            .onTapGesture {
                withAnimation {
                    isDisplayed = false
                }
            }
    }
}
