import SwiftUI
import AppSettings
import Constants
import ImpactGenerator
#if os(macOS)
import GRPCManager
#endif
import Theme
import UIComponents

public struct PrivacyAndDataView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var impactGenerator: ImpactGenerator
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
                logsSection()
                Spacer()
                    .frame(height: 24)
                diagnosticToolSection()
                Spacer()
                    .frame(height: 24)
                statisticsSection()
                Spacer()
                    .frame(height: 24)
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
            Color.Nym.background
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

    @ViewBuilder
    func logsSection() -> some View {
#if os(iOS)
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    isOn: $appSettings.isDebugLogsOn
                ),
                title: "settings.privacyAndData.enableDebugLogs".localizedString,
                systemImageName: "ladybug",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {}
            )
        )
        Spacer()
            .frame(height: 24)
#endif
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .arrow,
                title: "logs".localizedString,
                imageName: "logs",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {
                    navigateToLogs()
                }
            )
        )
    }

    func diagnosticToolSection() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.diagnosticTool".localizedString,
                systemImageName: "waveform.path.ecg.rectangle",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {
                    navigateToDiagnosticTool()
                }
            )
        )
    }

    func statisticsSection() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    isOn: $appSettings.isStatisticsEnabled
                ),
                title: "privacyData.anonymousStats".localizedString,
                multilineText: privacyMultilineText(),
                position: .init(isFirst: true, isLast: true),
                action: {}
            )
        )
#if os(macOS)
        .onChange(of: appSettings.isStatisticsEnabled) {
            enableMacOSNetworkStatsIfNeeded(with: appSettings.isStatisticsEnabled)
        }
#endif
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
                    isOn: $appSettings.isErrorReportingOn
                ),
                title: "privacyData.errorCrashReports".localizedString,
                multilineText: errorReportingMultilineText(),
                position: .init(isFirst: true, isLast: true),
                action: {}
            )
        )
#if os(macOS)
        .onChange(of: appSettings.isErrorReportingOn) {
            enableMacOSErrorReportingIfNeeded(with: appSettings.isErrorReportingOn)
        }
#endif
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

    func navigateToLogs() {
        impactGenerator.softImpact()
        path.append(SettingLink.logs)
    }

    func navigateToDiagnosticTool() {
        impactGenerator.softImpact()
        path.append(SettingLink.diagnosticTool)
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
