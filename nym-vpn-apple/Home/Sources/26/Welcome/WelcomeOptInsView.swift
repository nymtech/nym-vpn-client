import SwiftUI
import AppSettings
import Constants
import ImpactGenerator
import Theme
import UIComponents

public struct WelcomeOptInsView: View {

    @EnvironmentObject private var appSettings: AppSettings

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
            }
            cards
            continueButton
        }
        .padding(.horizontal, NymSpacing.component)
        .padding(.vertical, AuthLayout.verticalPadding)
        .frame(maxWidth: .infinity)
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

    var cards: some View {
        VStack(spacing: NymSpacing.small) {
            optInCard(
                title: "welcomeOptIns.stats.title".localizedString,
                linkTitle: "welcomeOptIns.stats.link".localizedString,
                linkURL: URL(string: Constants.anonymousStatsURL.rawValue),
                isOn: statisticsBinding
            )
            optInCard(
                title: "welcomeOptIns.error.title".localizedString,
                linkTitle: "welcomeOptIns.error.link".localizedString,
                linkURL: URL(string: Constants.sentryURL.rawValue),
                isOn: $appSettings.isErrorReportingOn
            )
        }
    }

    func optInCard(
        title: String,
        linkTitle: String,
        linkURL: URL?,
        isOn: Binding<Bool>
    ) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: NymSpacing.small) {
                Text(title)
                    .nymTextStyle(.bodyDefaultBold)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .frame(maxWidth: .infinity, alignment: .leading)
                Toggle("", isOn: isOn)
                    .labelsHidden()
                    .tint(Color.Nym.primary)
                    .accessibilityLabel(Text(title))
            }
            if let linkURL {
                Link(destination: linkURL) {
                    Text(linkTitle)
                        .nymTextStyle(.bodySmall)
                        .foregroundStyle(Color.Nym.info)
                        .underline()
                }
            } else {
                Text(linkTitle)
                    .nymTextStyle(.bodySmall)
                    .foregroundStyle(Color.Nym.info)
                    .underline()
            }
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 16)
                .fill(Color.Nym.surfaceAlt)
        )
    }

    var statisticsBinding: Binding<Bool> {
        Binding(
            get: {
                if welcomeStatisticsNeedsSeed(for: appSettings) {
                    return true
                }
                return appSettings.isStatisticsEnabled
            },
            set: { appSettings.isStatisticsEnabled = $0 }
        )
    }

    var continueButton: some View {
        NymButton("welcome.continue".localizedString, style: .primary) {
            if welcomeStatisticsNeedsSeed(for: appSettings) {
                appSettings.isStatisticsEnabled = true
            }
            ImpactGenerator.shared.softImpact()
            onContinue()
        }
    }
}

func welcomeStatisticsNeedsSeed(welcomeScreenDidDisplay: Bool, statisticsStored: Bool) -> Bool {
    !welcomeScreenDidDisplay && !statisticsStored
}

func welcomeStatisticsNeedsSeed(for appSettings: AppSettings) -> Bool {
    welcomeStatisticsNeedsSeed(
        welcomeScreenDidDisplay: appSettings.welcomeScreenDidDisplay,
        statisticsStored: UserDefaults.standard.object(forKey: AppSettingKey.statistics.rawValue) != nil
    )
}

#if DEBUG
#Preview {
    WelcomeOptInsView(onContinue: {})
        .environmentObject(AppSettings.shared)
        .background(Color.Nym.surface)
}
#endif
