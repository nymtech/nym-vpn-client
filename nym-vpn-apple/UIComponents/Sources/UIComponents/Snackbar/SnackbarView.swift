import SwiftUI
import AppSettings
import MessageModels
import Theme

public struct SnackbarView: View {
    private let appSettings: AppSettings
    private let message: SnackBarMessage?

    @Binding private var isDisplayed: Bool

    public init(
        isDisplayed: Binding<Bool>,
        message: SnackBarMessage?,
        appSettings: AppSettings
    ) {
        self._isDisplayed = isDisplayed
        self.message = message
        self.appSettings = appSettings
    }

    public var body: some View {
        VStack {
            if isDisplayed, let message {
                HStack(alignment: .center, spacing: 16) {
                    messageStyleImage()
                    messageText()
                    ctaActionText()
                    closeButton()
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 16)
                .frame(maxWidth: .infinity, minHeight: 35)
                .padding(.vertical, 8)
                .background(message.style.backgroundColor)
                .cornerRadius(10)
                .padding(.horizontal, 16)
                .padding(.top, appSettings.isSmallScreen ? 64 : 80) // CustomNavBarSize + 16
                .transition(.slide)
                .animation(.easeInOut(duration: 0.3), value: isDisplayed)
            }
            Spacer()
        }
    }
}

extension SnackbarView {
    @ViewBuilder
    func messageStyleImage() -> some View {
        if let message, let name = message.style.systemIconName {
            Image(systemName: name)
                .resizable()
                .foregroundStyle(message.style.iconColor)
                .aspectRatio(contentMode: .fit)
                .frame(width: 14, height: 14)
        }
    }

    @ViewBuilder
    func messageText() -> some View {
        if let message {
            Text(message.text)
                .foregroundColor(message.style.textColor)
                .font(.system(size: 14))
                .frame(alignment: .leading)
        }
    }

    @ViewBuilder
    func ctaActionText() -> some View {
        if let message, let ctaText = message.ctaText {
            Spacer()
            Text(ctaText)
                .foregroundColor(NymColor.accent)
                .textStyle(.Body.Medium.regular)
                .padding(8)
                .contentShape(Rectangle())
                .onTapGesture {
                    message.ctaAction?()
                }
                .accessibilityAction {
                    message.ctaAction?()
                }
        }
    }

    @ViewBuilder
    func closeButton() -> some View {
        if let message {
            Spacer()
            Image(systemName: "xmark")
                .resizable()
                .foregroundStyle(message.style.iconColor)
                .aspectRatio(contentMode: .fit)
                .frame(width: 14, height: 14)
                .padding(8)
                .contentShape(Rectangle())
                .onTapGesture {
                    withAnimation {
                        isDisplayed = false
                    }
                }
        }
    }
}
