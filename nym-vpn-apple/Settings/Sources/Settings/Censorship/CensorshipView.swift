import SwiftUI
import AppSettings
import FeatureFlagsManager
import Constants
import UIComponents
import Theme

public struct CensorshipView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var featureFlagsManager: FeatureFlagsManager
    @Binding private var path: NavigationPath

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)
            VStack(spacing: 0) {
                if featureFlagsManager.isQuicEnabled {
                    quicSection()
                    Spacer()
                        .frame(height: 24)
                }

                if featureFlagsManager.isDomainFrontingEnabled {
                    stealhApiSection()
                }
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
                        isOn: $appSettings.isCensorshipQuicEnabled,
                        action: { isOn in
                            // TODO: enable on macOS
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
        // TODO: update link
        let link = Constants.stealthConnectURL.rawValue
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
                        isOn: $appSettings.isCensorshipQuicEnabled,
                        action: { isOn in
                            // TODO: enable on macOS
                        }
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
}

// MARK: - Actions -
private extension CensorshipView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
