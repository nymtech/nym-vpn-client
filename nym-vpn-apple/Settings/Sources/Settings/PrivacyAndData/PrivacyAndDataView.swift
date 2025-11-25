import SwiftUI
import AppSettings
import Constants
#if os(macOS)
import GRPCManager
#endif
import Theme
import UIComponents

public struct PrivacyAndDataView: View {
    @EnvironmentObject private var appSettings: AppSettings
#if os(macOS)
    @EnvironmentObject private var grpcManager: GRPCManager
#endif
    @Binding private var path: NavigationPath
    @State private var isHovered = false
    @State private var hoveredId: Int?

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)
            VStack(spacing: 0) {
#if os(macOS)
                statisticsSection()
                Spacer()
                    .frame(height: 24)
#endif
                errorReportingSection()
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
private extension PrivacyAndDataView {
    func navbar() -> some View {
        CustomNavBar(
            title: "settings.privacyAndData".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    func statisticsSection() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    viewModel: ToggleViewModel(
                        isOn: $appSettings.isStatisticsEnabled,
                        action: { isOn in
#if os(macOS)
                            enableMacOSNetworkStatsIfNeeded(with: isOn)
#endif
                        }
                    )
                ),
                title: "privacyData.anonymousStats".localizedString,
                multilineText: privacyMultilineText(),
                position: .init(isFirst: true, isLast: true),
                action: {}
            )
        )
    }

    func privacyMultilineText() -> AttributedString? {
        let first  = "privacyData.anonymousStats.subtitle1".localizedString
        let second = "privacyData.anonymousStats.subtitle2".localizedString
        let third  = "privacyData.anonymousStats.subtitle3".localizedString
        let link   = Constants.anonymousStatsURL.rawValue
        let markdown = """
\(first)

\(second)

[\(third)](\(link))
"""

        let options = AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace)
        guard var text = try? AttributedString(markdown: markdown, options: options) else { return nil }
        if let range = text.range(of: third) {
            text[range].underlineStyle = .single
        }
        return text
    }

    func errorReportingSection() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    viewModel: ToggleViewModel(
                        isOn: $appSettings.isErrorReportingOn,
                        action: { isOn in
#if os(macOS)
                            enableMacOSErrorReportingIfNeeded(with: isOn)
#endif
                        }
                    )
                ),
                title: "privacyData.errorCrashReports".localizedString,
                multilineText: errorReportingMultilineText(),
                position: .init(isFirst: true, isLast: true),
                action: {}
            )
        )
    }

    func errorReportingMultilineText() -> AttributedString? {
        let first  = "privacyData.errorCrashReports.subtitle1".localizedString
        let second = "privacyData.errorCrashReports.subtitle2".localizedString
        let link   = Constants.sentryPrivacyURL.rawValue
        let markdown = """
\(first)

[\(second)](\(link))
"""

        let options = AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace)
        guard var text = try? AttributedString(markdown: markdown, options: options) else { return nil }
        if let range = text.range(of: second) {
            text[range].underlineStyle = .single
        }
        return text
    }
}

// MARK: - Actions -
private extension PrivacyAndDataView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}

#if os(macOS)
// MARK: - macOS actions
private extension PrivacyAndDataView {
    func enableMacOSErrorReportingIfNeeded(with isOn: Bool) {
        Task {
            try? await grpcManager.updateErrorReportingIfNeeded(with: isOn)
        }
    }

    func enableMacOSNetworkStatsIfNeeded(with isOn: Bool) {
        Task {
            try? await grpcManager.updateNetworkStatisticsIfNeeded(with: isOn)
        }
    }
}
#endif
