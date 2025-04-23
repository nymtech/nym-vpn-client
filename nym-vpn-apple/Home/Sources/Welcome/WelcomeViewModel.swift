import SwiftUI
import AppSettings
import UIComponents
import Theme

public final class WelcomeViewModel: ObservableObject {
    private var appSettings: AppSettings

    let titleText = "welcome.title"
    let subtitle1Text = "welcome.subtitle1"
    let subtitle2Text = "welcome.subtitle2"
    let sentryText = "welcome.sentry"
    let continueText = "welcome.continue"
    let disclaimerText = "welcome.disclaimer"

    public init(appSettings: AppSettings = AppSettings.shared) {
        self.appSettings = appSettings
    }

    func subtitleViewHorizontalPadding() -> CGFloat {
        appSettings.isSmallScreen ? 16 : 65
    }

    func sentryViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .toggle(
                viewModel: ToggleViewModel(
                    isOn: appSettings.isErrorReportingOn,
                    action: { [weak self] isOn in
                        self?.appSettings.isErrorReportingOn = isOn
                    }
                )
            ),
            title: "settings.anonymousErrorReports.title".localizedString,
            subtitle: "settings.anonymousErrorReports.subtitle".localizedString,
            imageName: "errorReport",
            position: .init(isFirst: true, isLast: true),
            action: {}
        )
    }

    func continueTapped() {
        appSettings.welcomeScreenDidDisplay = true
    }
}
