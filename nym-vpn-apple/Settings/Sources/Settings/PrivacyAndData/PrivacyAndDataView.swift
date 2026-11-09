import SwiftUI
import AppSettings
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
    @State private var infoContent: TelemetryInfoContent?

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
        .sheet(item: $infoContent) { content in
            TelemetryInfoSheet(content: content) {
                infoContent = nil
            }
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
        optInCard(
            icon: "chart.bar",
            title: "welcomeOptIns.connection.title".localizedString,
            subtitle: "welcomeOptIns.connection.subtitle".localizedString,
            isOn: $appSettings.isStatisticsEnabled,
            info: connectionInfo()
        )
#if os(macOS)
        .onChange(of: appSettings.isStatisticsEnabled) {
            enableMacOSNetworkStatsIfNeeded(with: appSettings.isStatisticsEnabled)
        }
#endif
    }

    func errorReportingSection() -> some View {
        optInCard(
            icon: "ladybug",
            title: "welcomeOptIns.monitoring.title".localizedString,
            subtitle: "welcomeOptIns.monitoring.subtitle".localizedString,
            isOn: $appSettings.isErrorReportingOn,
            info: monitoringInfo()
        )
#if os(macOS)
        .onChange(of: appSettings.isErrorReportingOn) {
            enableMacOSErrorReportingIfNeeded(with: appSettings.isErrorReportingOn)
        }
#endif
    }

    // Matches the onboarding opt-in card (WelcomeOptInsView). The info sheet and
    // its content are shared via UIComponents; only this small row layout is
    // repeated across the two modules.
    func optInCard(
        icon: String,
        title: String,
        subtitle: String,
        isOn: Binding<Bool>,
        info: TelemetryInfoContent
    ) -> some View {
        HStack(spacing: NymSpacing.small) {
            Image(systemName: icon)
                .foregroundStyle(Color.Nym.textSecondary)
                .frame(width: 24, height: 24)
                .accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 6) {
                    Text(title)
                        .nymTextStyle(.bodyDefaultBold)
                        .foregroundStyle(Color.Nym.textPrimary)
                    Button {
                        infoContent = info
                    } label: {
                        Image(systemName: "info.circle")
                            .foregroundStyle(Color.Nym.textSecondary)
                    }
                    .buttonStyle(.plain)
                    .accessibilityLabel(
                        Text(String(format: "welcomeOptIns.info.accessibility".localizedString, title))
                    )
                }
                Text(subtitle)
                    .nymTextStyle(.bodySmall)
                    .foregroundStyle(Color.Nym.textSecondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            Toggle("", isOn: isOn)
                .labelsHidden()
                .tint(Color.Nym.primary)
                .accessibilityLabel(Text(title))
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 16)
                .fill(Color.Nym.surfaceAlt)
        )
    }

    func connectionInfo() -> TelemetryInfoContent {
        .telemetry(
            id: "connection",
            sharesKey: "welcomeOptIns.connection.shares",
            neverKey: "welcomeOptIns.connection.never",
            handlingKey: "welcomeOptIns.connection.handling"
        )
    }

    func monitoringInfo() -> TelemetryInfoContent {
        .telemetry(
            id: "monitoring",
            sharesKey: "welcomeOptIns.monitoring.shares",
            neverKey: "welcomeOptIns.monitoring.never",
            handlingKey: "welcomeOptIns.monitoring.handling"
        )
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
