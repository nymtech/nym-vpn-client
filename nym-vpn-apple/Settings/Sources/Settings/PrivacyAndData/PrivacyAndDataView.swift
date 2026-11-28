import SwiftUI
import AppSettings
import ConfigurationManager
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
#if os(macOS)
    /// Set after a grace period, so the connect at launch doesn't flash "Daemon not running".
    @State private var isDaemonUnreachable = false
#endif

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
#if os(macOS)
                daemonDataSharingSection()
#else
                dataSharingSection(
                    statistics: $appSettings.isStatisticsEnabled,
                    errorReporting: $appSettings.isErrorReportingOn
                )
#endif
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
#if os(macOS)
        .task(id: grpcManager.isServing) {
            isDaemonUnreachable = false
            guard !grpcManager.isServing else { return }
            try? await Task.sleep(for: .seconds(5))
            guard !Task.isCancelled, !grpcManager.isServing else { return }
            isDaemonUnreachable = true
        }
#endif
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
        diagnosticsSection()
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

#if os(iOS)
    func diagnosticsSection() -> some View {
        // TestFlight builds always log at debug level, so show the toggle locked on.
        let isTestFlight = ConfigurationManager.shared.isTestFlight
        return TelemetryOptInCard(
            icon: "stethoscope",
            title: "welcomeOptIns.diagnostics.title".localizedString,
            subtitle: isTestFlight
                ? "welcomeOptIns.diagnostics.testFlight".localizedString
                : "welcomeOptIns.diagnostics.subtitle".localizedString,
            isOn: isTestFlight ? .constant(true) : $appSettings.isDebugLogsOn,
            isDisabled: isTestFlight,
            onInfo: { infoContent = diagnosticsInfo() }
        )
    }

    func diagnosticsInfo() -> TelemetryInfoContent {
        .telemetry(
            id: "diagnostics",
            sharesKey: "welcomeOptIns.diagnostics.shares",
            neverKey: nil,
            handlingKey: "welcomeOptIns.diagnostics.handling"
        )
    }
#endif

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

    @ViewBuilder
    func dataSharingSection(statistics: Binding<Bool>, errorReporting: Binding<Bool>) -> some View {
        TelemetryOptInCard(
            icon: "chart.bar",
            title: "welcomeOptIns.connection.title".localizedString,
            subtitle: "welcomeOptIns.connection.subtitle".localizedString,
            isOn: statistics,
            onInfo: { infoContent = connectionInfo() }
        )
        Spacer()
            .frame(height: 24)
        TelemetryOptInCard(
            icon: "ladybug",
            title: "welcomeOptIns.monitoring.title".localizedString,
            subtitle: "welcomeOptIns.monitoring.subtitle".localizedString,
            note: TelemetryOptInCard.errorReportingRestartNote,
            isOn: errorReporting,
            onInfo: { infoContent = monitoringInfo() }
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
// MARK: - Data-sharing (macOS) -
// nym-vpnd owns these settings, so without it, or before onboarding has asked, there are no
// toggles to show.
private extension PrivacyAndDataView {
    @ViewBuilder
    func daemonDataSharingSection() -> some View {
        if grpcManager.isServing && appSettings.welcomeScreenDidDisplay {
            dataSharingSection(
                statistics: consentBinding(.networkStatistics, \.isStatisticsEnabled),
                errorReporting: consentBinding(.errorReporting, \.isErrorReportingOn)
            )
        } else if isDaemonUnreachable {
            SettingsListItem(
                viewModel: SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "daemonInstall.title".localizedString,
                    systemImageName: "exclamationmark.triangle",
                    position: SettingsListItemPosition(isFirst: true, isLast: true),
                    action: {
                        impactGenerator.softImpact()
                        path.append(SettingLink.daemonEnable)
                    }
                )
            )
        }
    }

    func consentBinding(_ setting: TelemetryConsentSetting, _ value: KeyPath<AppSettings, Bool>) -> Binding<Bool> {
        Binding(
            get: { appSettings[keyPath: value] },
            set: { ConfigurationManager.shared.userChangedTelemetryConsent(setting, to: $0) }
        )
    }
}
#endif
