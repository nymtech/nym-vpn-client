#if os(iOS)
import SwiftUI
import AppSettings
import ConfigurationManager
import UIComponents

/// The debug-logging toggle ("Your troubleshooting detail"), shared by the
/// onboarding opt-in screen and Settings › Data, privacy & logs.
public struct DiagnosticsOptInCard: View {
    @EnvironmentObject private var appSettings: AppSettings
    private let onInfo: (TelemetryInfoContent) -> Void

    public init(onInfo: @escaping (TelemetryInfoContent) -> Void) {
        self.onInfo = onInfo
    }

    public var body: some View {
        // TestFlight builds always log at debug level, so show the toggle locked on.
        let isTestFlight = ConfigurationManager.shared.isTestFlight
        TelemetryOptInCard(
            icon: "stethoscope",
            title: "welcomeOptIns.diagnostics.title".localizedString,
            subtitle: isTestFlight
                ? "welcomeOptIns.diagnostics.testFlight".localizedString
                : "welcomeOptIns.diagnostics.subtitle".localizedString,
            isOn: isTestFlight ? .constant(true) : $appSettings.isDebugLogsOn,
            isLocked: isTestFlight,
            onInfo: { onInfo(Self.info) }
        )
    }

    static var info: TelemetryInfoContent {
        .telemetry(
            id: "diagnostics",
            sharesKey: "welcomeOptIns.diagnostics.shares",
            neverKey: nil,
            handlingKey: "welcomeOptIns.diagnostics.handling"
        )
    }
}
#endif
