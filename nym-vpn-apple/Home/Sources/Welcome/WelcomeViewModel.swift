import SwiftUI
import AppSettings
import Constants
import ImpactGenerator
import UIComponents
import Theme

@MainActor public final class WelcomeViewModel: ObservableObject {
    private var appSettings: AppSettings

    let titleText = "welcome.title".localizedString
    let subtitle1Text = "welcome.subtitle1".localizedString
    let subtitle2Text = "welcome.subtitle2".localizedString
    let subtitle3Text = "welcome.subtitle3".localizedString
    let privacyPolicy1Text = "welcome.privacyPolicy1".localizedString
    let privacyPolicy2Text = "welcome.privacyPolicy2".localizedString
    let termsOfUse = "welcome.termsOfUse".localizedString
    let privacyPolicy = "welcome.privacyPolicy".localizedString
    let sentryText = "welcome.sentry".localizedString
    let continueText = "welcome.continue".localizedString

    public init(appSettings: AppSettings) {
        self.appSettings = appSettings
    }

    func subtitleViewHorizontalPadding() -> CGFloat {
        appSettings.isSmallScreen ? 0 : 49
    }

    func sentryViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .toggle(viewModel: ToggleViewModel(isOn: appSettings.$isErrorReportingOn)),
            title: "settings.anonymousErrorReports.title".localizedString,
            subtitle: "settings.anonymousErrorReports.subtitle".localizedString,
            imageName: "errorReport",
            position: .init(isFirst: true, isLast: false),
            action: {}
        )
    }

    func statisticsViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .toggle(viewModel: ToggleViewModel(isOn: appSettings.$isStatisticsEnabled)),
            title: "welcome.analytics".localizedString,
            imageName: "statistics",
            position: .init(isFirst: false, isLast: true),
            action: {}
        )
    }

    func subtitleAttributedString() -> AttributedString? {
        let options = AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace)
        return try? AttributedString(
            markdown: "\(subtitle1Text) [\(sentryText)](\(Constants.sentryURL.rawValue))\(subtitle2Text)\n\n\(subtitle3Text)",
            options: options
        )
    }

    func privacyPolicyAttributedString() -> AttributedString? {
        try? AttributedString(markdown: "\(privacyPolicy1Text) [\(termsOfUse)](\(Constants.termsOfUseURL.rawValue)) \(privacyPolicy2Text) [\(privacyPolicy)](\(Constants.privacyPolicyURL.rawValue))")
    }

    func continueTapped() {
        ImpactGenerator.shared.impact()
        appSettings.welcomeScreenDidDisplay = true
    }
}
