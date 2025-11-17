import SwiftUI
import AppSettings
import ConnectionManager
import FeatureFlagsManager
import Constants
import UIComponents
import Theme

public struct CensorshipView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var featureFlagsManager: FeatureFlagsManager
    @Binding private var path: NavigationPath
    @State private var isConfirmationDisplayed = false

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)
            VStack(spacing: 0) {
                quicSection()
                Spacer()
                    .frame(height: 24)
                stealhApiSection()
            }
            .frame(maxWidth: MagicNumbers.maxWidth)
            .padding(.horizontal, 16)
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
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

    // MARK: - QUIC -
    func quicSection() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    viewModel: ToggleViewModel(
                        isOn: $appSettings.isQuicEnabled,
                        controlInAlert: true,
                        isDisplayingAlert: $isConfirmationDisplayed,
                        action: { isOn in
                            guard connectionManager.currentTunnelStatus == .connected ||
                                    connectionManager.currentTunnelStatus == .connecting
                            else {
                                appSettings.isQuicEnabled.toggle()
                                return
                            }
                            isConfirmationDisplayed = true
                        }
                    )
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
            text[range].foregroundColor = NymColor.primary
        }
        return text
    }

    // MARK: - Stealh API -
    func stealhApiSection() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    viewModel: ToggleViewModel(
                        isOn: .constant(true),
                        isDisabled: true
                    )
                ),
                title: "censorship.stealhapi.title".localizedString,
                multilineText: stealhApiMultilineText(),
                position: .init(isFirst: true, isLast: true),
                action: {}
            )
        )
    }

    func stealhApiMultilineText() -> AttributedString? {
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
            text[range].foregroundColor = NymColor.primary
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
                appSettings.isQuicEnabled.toggle()
                appSettings.shouldReconnect = true
                isConfirmationDisplayed = false
                path = .init()
            },
            noAction: {
                appSettings.isQuicEnabled.toggle()
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
