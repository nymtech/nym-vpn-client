import SwiftUI
import AppSettings
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
