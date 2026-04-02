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
                Group {
                    switch message.style {
                    case .expiry:
                        expiryContent(message)
                    case .passphrase:
                        passphraseContent(message)
                    default:
                        defaultContent(message)
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 16)
                .frame(maxWidth: .infinity, minHeight: 35)
                .padding(.vertical, message.style == .passphrase ? 16 : 8)
                .background(message.style.backgroundColor)
                .cornerRadius(10)
                .padding(.horizontal, 16)
                .padding(.top, appSettings.isSmallScreen ? 64 : 80)
                .transition(.move(edge: .top).combined(with: .opacity))
                .animation(.easeInOut(duration: 0.3), value: isDisplayed)
            }
            Spacer()
        }
    }
}

extension SnackbarView {
    @ViewBuilder
    func defaultContent(_ message: SnackBarMessage) -> some View {
        HStack(alignment: .center, spacing: 16) {
            messageStyleImage(message)
            messageText(message)
            if let ctaText = message.ctaText {
                Spacer()
                ctaButton(ctaText, action: message.ctaAction)
            }
            if message.style.showsCloseButton {
                closeButton()
            }
        }
    }

    @ViewBuilder
    func expiryContent(_ message: SnackBarMessage) -> some View {
        HStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 2) {
                Text(message.text)
                    .textStyle(.Body.Medium.regular)
                    .foregroundStyle(NymColor.primary)

                if let subtitle = message.subtitle {
                    Text(subtitle)
                        .textStyle(.Body.Medium.regular)
                        .foregroundStyle(NymColor.gray1)
                }
            }
            .layoutPriority(1)
            Spacer()
            if let ctaText = message.ctaText {
                ctaButton(ctaText, action: message.ctaAction)
            }
            closeButton()
        }
    }

    @ViewBuilder
    func passphraseContent(_ message: SnackBarMessage) -> some View {
        HStack(spacing: 8) {
            Text(message.text)
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Medium.regular)
                .layoutPriority(1)
            Spacer()
            if let ctaText = message.ctaText {
                Text(ctaText)
                    .foregroundStyle(NymColor.accent)
                    .textStyle(.Body.Medium.bold)
                    .multilineTextAlignment(.trailing)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        message.ctaAction?()
                    }
                    .accessibilityAction {
                        message.ctaAction?()
                    }
            }
        }
    }
}

extension SnackbarView {
    @ViewBuilder
    func messageStyleImage(_ message: SnackBarMessage) -> some View {
        if let name = message.style.systemIconName {
            Image(systemName: name)
                .resizable()
                .foregroundStyle(message.style.iconColor)
                .aspectRatio(contentMode: .fit)
                .frame(width: 14, height: 14)
        }
    }

    @ViewBuilder
    func messageText(_ message: SnackBarMessage) -> some View {
        Text(message.text)
            .foregroundColor(message.style.textColor)
            .font(.system(size: 14))
            .frame(alignment: .leading)
    }

    @ViewBuilder
    func ctaButton(_ text: String, action: (() -> Void)?) -> some View {
        Text(text)
            .foregroundColor(NymColor.accent)
            .textStyle(.Body.Medium.regular)
            .multilineTextAlignment(.center)
            .padding(8)
            .contentShape(Rectangle())
            .onTapGesture {
                action?()
            }
            .accessibilityAction {
                action?()
            }
    }

    @ViewBuilder
    func closeButton() -> some View {
        Image(systemName: "xmark")
            .resizable()
            .foregroundStyle(NymColor.primary)
            .aspectRatio(contentMode: .fit)
            .frame(width: 12, height: 12)
            .padding(8)
            .contentShape(Rectangle())
            .onTapGesture {
                withAnimation {
                    isDisplayed = false
                }
            }
    }
}
