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
                // On macOS the daemon holds these values; without it there is nothing true to show.
                if isDaemonUnavailable {
                    daemonNotRunningSection()
                } else {
                    statisticsSection()
                    Spacer()
                        .frame(height: 24)
                    errorReportingSection()
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

    func statisticsSection() -> some View {
        TelemetryOptInCard(
            icon: "chart.bar",
            title: "welcomeOptIns.connection.title".localizedString,
            subtitle: "welcomeOptIns.connection.subtitle".localizedString,
            isOn: statisticsBinding,
            onInfo: { infoContent = connectionInfo() }
        )
    }

    func errorReportingSection() -> some View {
        TelemetryOptInCard(
            icon: "ladybug",
            title: "welcomeOptIns.monitoring.title".localizedString,
            subtitle: "welcomeOptIns.monitoring.subtitle".localizedString,
            note: errorReportingNote,
            isOn: errorReportingBinding,
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

// MARK: - Data-sharing state -
// On macOS nym-vpnd is the source of truth: the toggles show its values (mirrored into
// AppSettings by ConfigurationManager) and send only the user's own changes.
private extension PrivacyAndDataView {
    var isDaemonUnavailable: Bool {
#if os(macOS)
        return !grpcManager.isServing
#else
        return false
#endif
    }

    var statisticsBinding: Binding<Bool> {
#if os(macOS)
        return Binding(
            get: { appSettings.isStatisticsEnabled },
            set: { isOn in
                Task { await ConfigurationManager.shared.setNetworkStatistics(isOn) }
            }
        )
#else
        return $appSettings.isStatisticsEnabled
#endif
    }

    var errorReportingBinding: Binding<Bool> {
#if os(macOS)
        return Binding(
            get: { appSettings.isErrorReportingOn },
            set: { isOn in
                Task { await ConfigurationManager.shared.setErrorReporting(isOn) }
            }
        )
#else
        return $appSettings.isErrorReportingOn
#endif
    }

    /// The daemon starts error reporting only when it launches.
    var errorReportingNote: String? {
#if os(macOS)
        return "settings.privacyAndData.errorReportingRestartNote".localizedString
#else
        return nil
#endif
    }
}

// MARK: - Daemon not running (macOS) -
private extension PrivacyAndDataView {
    @ViewBuilder
    func daemonNotRunningSection() -> some View {
#if os(macOS)
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
#endif
    }
}
