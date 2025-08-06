import SwiftUI
import AppSettings
#if os(iOS)
import ImpactGenerator
#endif
import UIComponents
import Theme

public final class WelcomeViewModel: ObservableObject {
    private var appSettings: AppSettings

    let titleText = "welcome.title".localizedString
    let subtitle1Text = "welcome.subtitle1".localizedString
    let subtitle2Text = "welcome.subtitle2".localizedString
    let sentryText = "welcome.sentry".localizedString
    let continueText = "welcome.continue".localizedString
    let disclaimerText = "welcome.disclaimer".localizedString

    public init(appSettings: AppSettings = AppSettings.shared) {
        self.appSettings = appSettings
    }

    func subtitleViewHorizontalPadding() -> CGFloat {
        appSettings.isSmallScreen ? 16 : 65
    }

    func sentryViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .toggle(viewModel: ToggleViewModel(isOn: appSettings.$isErrorReportingOn)),
            title: "settings.anonymousErrorReports.title".localizedString,
            subtitle: "settings.anonymousErrorReports.subtitle".localizedString,
            imageName: "errorReport",
            position: .init(isFirst: true, isLast: true),
            action: {}
        )
    }

    func continueTapped() {
#if os(iOS)
        ImpactGenerator.shared.impact()
#endif
        appSettings.welcomeScreenDidDisplay = true
    }
}
