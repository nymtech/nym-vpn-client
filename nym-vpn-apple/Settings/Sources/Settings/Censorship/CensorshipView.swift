import SwiftUI
import AppSettings
import ConnectionManager
import Constants
import UIComponents
import Theme

public struct CensorshipView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var connectionManager: ConnectionManager
    @Binding private var path: NavigationPath
    @State private var isConfirmationDisplayed = false

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)
            VStack(spacing: 0) {
                ScrollView {
                    subtitleSection()
                    quicSection()
                    Spacer()
                        .frame(height: 24)
                    stealthApiSection()
                }
                .scrollIndicators(.never)
            }
            .frame(maxWidth: MagicNumbers.maxWidth)
            .padding(.horizontal, 16)
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
        .overlay {
            confirmationOnlineOverlay()
        }
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

// MARK: - Views -
private extension CensorshipView {
    func navbar() -> some View {
        CustomNavBar(
            title: "settings.censorship.title".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    func subtitleSection() -> some View {
        Text("censorship.subtitle".localizedString)
            .nymTextStyle(.bodyDefault)
            .foregroundStyle(Color.Nym.textSecondary)
            .padding(.bottom, 12)
    }

    // MARK: - QUIC -
    func quicSection() -> some View {
        let quicBinding = Binding<Bool>(
            get: { appSettings.isQuicEnabled },
            set: { _ in
                guard connectionManager.currentTunnelStatus == .connected ||
                        connectionManager.currentTunnelStatus == .connecting
                else {
                    connectionManager.setBridges(!appSettings.isQuicEnabled)
                    return
                }
                isConfirmationDisplayed = true
            }
        )

        return SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    isOn: quicBinding
                ),
                title: "censorship.quic.title".localizedString,
                multilineText: quicMultilineText(),
                position: .init(isFirst: true, isLast: true),
                action: {}
            )
        )
    }

    func quicMultilineText() -> AttributedString? {
        let first = "censorship.quic.subtitle1".localizedString
        let second = "censorship.quic.subtitle2".localizedString
        let link = Constants.quicURL.rawValue
        let linkText = "censorship.quic.link".localizedString
        let markdown = """
\(first)

\(second)

[\(linkText)](\(link))
"""

        let options = AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace)
        guard var text = try? AttributedString(markdown: markdown, options: options) else { return nil }
        if let range = text.range(of: linkText) {
            text[range].underlineStyle = .single
            text[range].foregroundColor = Color.Nym.textPrimary
        }
        return text
    }

    // MARK: - Stealth API -
    func stealthApiSection() -> some View {
        let stealthBinding = Binding<Bool>(
            get: { appSettings.isStealthApiEnabled },
            set: { _ in connectionManager.setStealthApiEnabled(!appSettings.isStealthApiEnabled) }
        )

        return SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    isOn: stealthBinding
                ),
                title: "censorship.stealhapi.title".localizedString,
                multilineText: stealthApiMultilineText(),
                position: .init(isFirst: true, isLast: true),
                action: {}
            )
        )
    }

    func stealthApiMultilineText() -> AttributedString? {
        let first = "censorship.stealhapi.subtitle1".localizedString
        // TODO: update link
        let link = Constants.stealhApiConnectURL.rawValue
        let linkText = "censorship.stealhapi.link".localizedString
        let markdown = """
\(first)

[\(linkText)](\(link))
"""

        let options = AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace)
        guard var text = try? AttributedString(markdown: markdown, options: options) else { return nil }
        if let range = text.range(of: linkText) {
            text[range].underlineStyle = .single
            text[range].foregroundColor = Color.Nym.textPrimary
        }
        return text
    }

    var overlayConfiguration: ActionDialogConfiguration {
        let alertTitle = appSettings.isQuicEnabled
        ? "censorship.quic.disable.alert.title".localizedString
        : "censorship.quic.enable.alert.title".localizedString

        let subtitle = "censorship.quic.disable.alert.subtitle".localizedString
        + "\n\n"
        + "censorship.quic.disable.alert.subtitle1".localizedString

        let yesTitle = appSettings.isQuicEnabled
        ? "censorship.quic.disableAndReconnect".localizedString
        : "censorship.quic.enableAndReconnect".localizedString

        let noTitle = appSettings.isQuicEnabled
        ? "censorship.quic.offAndNext".localizedString
        : "censorship.quic.onAndNext".localizedString

        return ActionDialogConfiguration(
            systemIconImageName: "shippingbox",
            titleLocalizedString: alertTitle,
            subtitleLocalizedString: subtitle,
            yesLocalizedString: yesTitle,
            noLocalizedString: noTitle,
            yesAction: {
                connectionManager.setBridges(!appSettings.isQuicEnabled)
                isConfirmationDisplayed = false
                path = .init()
            },
            noAction: {
                connectionManager.setBridges(!appSettings.isQuicEnabled)
                isConfirmationDisplayed = false
            },
            verticalButtonsLayout: true
        )
    }

    @ViewBuilder
    func confirmationOnlineOverlay() -> some View {
        if isConfirmationDisplayed {
            ActionDialogView(
                viewModel: ActionDialogViewModel(
                    isDisplayed: $isConfirmationDisplayed,
                    configuration: overlayConfiguration,
                    impactGenerator: .shared
                )
            )
            .transition(.opacity)
            .animation(.easeInOut, value: isConfirmationDisplayed)
        }
    }
}

// MARK: - Actions -
private extension CensorshipView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
