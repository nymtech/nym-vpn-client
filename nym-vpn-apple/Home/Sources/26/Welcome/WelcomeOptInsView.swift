import SwiftUI
import AppSettings
import Constants
import ImpactGenerator
import Theme
import UIComponents

public struct WelcomeOptInsView: View {

    @AppStorage(AppSettingKey.statistics.rawValue)
    private var isStatisticsEnabled = false

    @AppStorage(AppSettingKey.errorReporting.rawValue)
    private var isErrorReportingOn = false

    @EnvironmentObject private var appSettings: AppSettings

    @State private var infoContent: TelemetryInfoContent?

    private let onContinue: () -> Void

    public init(onContinue: @escaping () -> Void) {
        self.onContinue = onContinue
    }

    public var body: some View {
        VStack(spacing: NymSpacing.large) {
            logo
            VStack(spacing: AuthLayout.stackSpacing) {
                heading
                subtitle
                privacyLink
            }
            cards
            continueButton
        }
        .padding(.horizontal, NymSpacing.component)
        .padding(.vertical, AuthLayout.verticalPadding)
        .frame(maxWidth: .infinity)
        // SentryManager observes this publisher; AppStorage alone would not reach it until relaunch.
        .onChange(of: isErrorReportingOn) {
            appSettings.isErrorReportingOnPublisher = isErrorReportingOn
        }
        .sheet(item: $infoContent) { content in
            TelemetryInfoSheet(content: content) {
                infoContent = nil
            }
        }
    }
}

private extension WelcomeOptInsView {
    var logo: some View {
        GenericImage(imageName: "logoText")
            .frame(width: 100, height: 27)
            .accessibilityHidden(true)
    }

    var heading: some View {
        Text("welcomeOptIns.title".localizedString)
            .nymTextStyle(.titleScreen)
            .foregroundStyle(Color.Nym.textPrimary)
            .multilineTextAlignment(.center)
    }

    var subtitle: some View {
        Text("welcomeOptIns.subtitle".localizedString)
            .nymTextStyle(.bodyDefault)
            .foregroundStyle(Color.Nym.textSecondary)
            .multilineTextAlignment(.center)
            .padding(.horizontal, NymSpacing.component)
    }

    @ViewBuilder
    var privacyLink: some View {
        if let url = URL(string: Constants.privacyPolicyURL.rawValue) {
            Link(destination: url) {
                Text("welcomeOptIns.privacyLink".localizedString)
                    .nymTextStyle(.bodySmall)
                    .foregroundStyle(Color.Nym.info)
                    .underline()
            }
        }
    }

    var cards: some View {
        VStack(spacing: NymSpacing.small) {
            TelemetryOptInCard(
                icon: "chart.bar",
                title: "welcomeOptIns.connection.title".localizedString,
                subtitle: "welcomeOptIns.connection.subtitle".localizedString,
                isOn: $isStatisticsEnabled,
                onInfo: { infoContent = connectionInfo() }
            )
            TelemetryOptInCard(
                icon: "ladybug",
                title: "welcomeOptIns.monitoring.title".localizedString,
                subtitle: "welcomeOptIns.monitoring.subtitle".localizedString,
                isOn: $isErrorReportingOn,
                onInfo: { infoContent = monitoringInfo() }
            )
        }
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

    var continueButton: some View {
        NymButton("welcome.continue".localizedString, style: .primary) {
            ImpactGenerator.shared.softImpact()
            onContinue()
        }
    }
}

#if DEBUG
#Preview {
    WelcomeOptInsView(onContinue: {})
        .environmentObject(AppSettings.shared)
        .background(Color.Nym.surface)
}
#endif
