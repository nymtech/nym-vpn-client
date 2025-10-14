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

    @Published public private(set) var subtitleAttributed: AttributedString?
    @Published public private(set) var privacyPolicyAttributed: AttributedString?

    public init(appSettings: AppSettings) {
        self.appSettings = appSettings
        precomputeAttributedStrings()
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

    func subtitleAttributedString() -> AttributedString? { subtitleAttributed }
    func privacyPolicyAttributedString() -> AttributedString? { privacyPolicyAttributed }

    func continueTapped() {
        ImpactGenerator.shared.impact()
        appSettings.welcomeScreenDidDisplay = true
    }
}

private extension WelcomeViewModel {
    func precomputeAttributedStrings() {
        let s1 = subtitle1Text
        let s2 = subtitle2Text
        let s3 = subtitle3Text
        let sentry = sentryText
        let terms = termsOfUse
        let pp1 = privacyPolicy1Text
        let pp2 = privacyPolicy2Text
        let pp = privacyPolicy
        let termsURL = Constants.termsOfUseURL.rawValue
        let privacyURL = Constants.privacyPolicyURL.rawValue
        let sentryURL = Constants.sentryURL.rawValue

        Task.detached(priority: .low) { [weak self] in
            guard let self else { return }
            let options = AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace)

            let subtitleMarkdown =
            "\(s1) [\(sentry)](\(sentryURL))\(s2)\n\n\(s3)"
            let subtitle = try? AttributedString(markdown: subtitleMarkdown, options: options)

            let privacyMarkdown =
            "\(pp1) [\(terms)](\(termsURL)) \(pp2) [\(pp)](\(privacyURL))"
            let privacy = try? AttributedString(markdown: privacyMarkdown, options: options)

            await MainActor.run {
                self.subtitleAttributed = subtitle
                self.privacyPolicyAttributed = privacy
            }
        }
    }
}
