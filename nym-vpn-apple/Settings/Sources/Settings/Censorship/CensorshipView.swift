import SwiftUI
import AppSettings
import ConnectionManager
import ConnectionTypes
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

    // MARK: - Bridge mode -
    func quicSection() -> some View {
        let modeBinding = Binding<BridgeModeSetting>(
            get: { appSettings.bridgeMode },
            set: { connectionManager.setBridgeMode($0) }
        )

        return SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .empty,
                title: "censorship.quic.title".localizedString,
                badge: "2hopMixnetTitle".localizedString,
                position: .init(isFirst: true, isLast: true),
                action: {}
            ),
            customContent: {
                VStack(alignment: .leading, spacing: 12) {
                    Text(markdown("censorship.quic.subtitle".localizedString))
                        .nymTextStyle(.bodySmall)
                        .foregroundStyle(Color.Nym.textSecondary)
                    BridgeModeSelector(selection: modeBinding)
                    VStack(alignment: .leading, spacing: 16) {
                        Text(markdown("censorship.bridge.\(appSettings.bridgeMode.rawValue)".localizedString))
                            .foregroundStyle(Color.Nym.textSecondary)
                        Text("censorship.bridge.reconnect".localizedString)
                            .foregroundStyle(Color.Nym.warning)
                        learnMoreLink(
                            text: "censorship.quic.link".localizedString,
                            url: Constants.antiCensorshipBridgesURL.rawValue
                        )
                    }
                    .nymTextStyle(.bodySmall)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, NymSpacing.large)
                .padding(.bottom, 18)
            },
            combineAccessibilityChildren: false,
            compactTitle: true
        )
    }

    func markdown(_ string: String) -> AttributedString {
        let options = AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace)
        guard var text = try? AttributedString(markdown: string, options: options) else {
            return AttributedString(string)
        }
        for run in text.runs where run.inlinePresentationIntent?.contains(.stronglyEmphasized) == true {
            text[run.range].font = Font.Nym.bodySmallBold
        }
        return text
    }

    func learnMoreLink(text: String, url: String) -> some View {
        Text(markdown("[\(text)](\(url))"))
            .underline()
            .tint(Color.Nym.textPrimary)
            .foregroundStyle(Color.Nym.textPrimary)
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

private extension BridgeModeSetting {
    var title: String {
        switch self {
        case .off:
            "general.off".localizedString
        case .auto:
            "censorship.bridge.auto.label".localizedString
        case .on:
            "general.on".localizedString
        }
    }
}

private struct BridgeModeSelector: View {
    @Binding var selection: BridgeModeSetting
    @Namespace private var thumbNamespace

    var body: some View {
        HStack(spacing: 0) {
            ForEach(BridgeModeSetting.allCases, id: \.self) { mode in
                let isSelected = mode == selection
                Button {
                    withAnimation(.spring(response: 0.4, dampingFraction: 0.78)) {
                        selection = mode
                    }
                } label: {
                    Text(mode.title)
                        .nymTextStyle(.bodySmall)
                        .foregroundStyle(isSelected ? Color.Nym.primaryText : Color.Nym.textPrimary)
                        .frame(maxWidth: .infinity, minHeight: 40)
                        .background {
                            if isSelected {
                                thumb.matchedGeometryEffect(id: "thumb", in: thumbNamespace)
                            }
                        }
                        .contentShape(Capsule())
                }
                .buttonStyle(.plain)
                .accessibilityAddTraits(isSelected ? .isSelected : [])
            }
        }
        .padding(4)
        .background { track }
        .sensoryFeedback(.selection, trigger: selection)
    }

    @ViewBuilder private var thumb: some View {
        if #available(iOS 26, macOS 26, *) {
            Color.clear.glassEffect(.regular.tint(Color.Nym.primary).interactive(), in: Capsule())
        } else {
            Capsule().fill(Color.Nym.primary).shadow(color: .black.opacity(0.25), radius: 4, y: 2)
        }
    }

    @ViewBuilder private var track: some View {
        if #available(iOS 26, macOS 26, *) {
            Color.clear.glassEffect(.regular, in: Capsule())
        } else {
            Capsule().fill(Color.Nym.surfaceAlt)
        }
    }
}

// MARK: - Actions -
private extension CensorshipView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
