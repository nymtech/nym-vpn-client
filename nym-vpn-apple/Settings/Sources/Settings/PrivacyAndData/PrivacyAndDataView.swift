import SwiftUI
import AppSettings
import ImpactGenerator
#if os(iOS)
import ConfigurationManager
#elseif os(macOS)
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
                statisticsSection()
                Spacer()
                    .frame(height: 24)
                errorReportingSection()
                Spacer()
                    .frame(height: 24)
#if os(iOS)
                diagnosticsSection()
                Spacer()
                    .frame(height: 24)
#endif
                logsSection()
                Spacer()
                    .frame(height: 24)
                diagnosticToolSection()
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

    func logsSection() -> some View {
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
        GenericButton(title: "settings.diagnosticTool.run".localizedString)
            .onTapGesture {
                navigateToDiagnosticTool()
            }
            .accessibilityAction {
                navigateToDiagnosticTool()
            }
    }

    func statisticsSection() -> some View {
        TelemetryOptInCard(
            icon: "chart.bar",
            title: "welcomeOptIns.connection.title".localizedString,
            subtitle: "welcomeOptIns.connection.subtitle".localizedString,
            isOn: $appSettings.isStatisticsEnabled,
            onInfo: { infoContent = connectionInfo() }
        )
#if os(macOS)
        .onChange(of: appSettings.isStatisticsEnabled) {
            enableMacOSNetworkStatsIfNeeded(with: appSettings.isStatisticsEnabled)
        }
#endif
    }

    func errorReportingSection() -> some View {
        TelemetryOptInCard(
            icon: "ladybug",
            title: "welcomeOptIns.monitoring.title".localizedString,
            subtitle: "welcomeOptIns.monitoring.subtitle".localizedString,
            isOn: $appSettings.isErrorReportingOn,
            onInfo: { infoContent = monitoringInfo() }
        )
#if os(macOS)
        .onChange(of: appSettings.isErrorReportingOn) {
            enableMacOSErrorReportingIfNeeded(with: appSettings.isErrorReportingOn)
        }
#endif
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
