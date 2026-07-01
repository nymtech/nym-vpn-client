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
                connectionManager.setBridges(!appSettings.isQuicEnabled)
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

}

// MARK: - Actions -
private extension CensorshipView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
