import SwiftUI
import AppSettings
import Constants
import Device
import ImpactGenerator
import Theme
import UIComponents

public struct TechnicalOptInsView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @Binding private var path: NavigationPath

    public var body: some View {
        ZStack {
            NymColor.background
                .ignoresSafeArea()

            VStack(spacing: 0) {
                Spacer()
                titleView()
                subtitleView()
                settingsList()
                Spacer()
                    .frame(height: 24)
                continueButton()
                    .padding(.bottom, 24)
            }
            .padding(.horizontal, 16)
            .frame(minWidth: 375, maxWidth: Device.type == .ipad ? 450 : 500)
            .navigationBarBackButtonHidden()
        }
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

private extension TechnicalOptInsView {
    @ViewBuilder
    func titleView() -> some View {
        Text("welcome.title".localizedString)
            .textStyle(.Headline.Large.regular)
            .multilineTextAlignment(.center)
        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func subtitleView() -> some View {
        Text("welcome.subtitle1".localizedString)
            .textStyle(.Body.Medium.regular)
            .tint(NymColor.accent)
            .foregroundStyle(NymColor.gray1)
            .multilineTextAlignment(.center)
            .padding(.horizontal, appSettings.isSmallScreen ? 0 : 24)
        Spacer()
    }

    func settingsList() -> some View {
        SettingsList(
            viewModel: SettingsListViewModel(
                sections: [SettingsSection(kind: TechnicalOptInsSectionKind.main, viewModels: items)]
            )
        )
    }

    var items: [SettingsListItemViewModel] {
        var result: [SettingsListItemViewModel] = []
        #if os(iOS)
        result.append(debugLogsViewModel())
        #endif
        result.append(sentryViewModel())
        #if os(macOS)
        result.append(statisticsViewModel())
        #endif
        return result
    }

    @ViewBuilder
    func continueButton() -> some View {
        GenericButton(title: "welcome.continue".localizedString)
            .onTapGesture {
                continueTapped()
            }
            .accessibilityAction {
                continueTapped()
            }
    }
}

private extension TechnicalOptInsView {
    func debugLogsViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .toggle(
                isOn: $appSettings.isDebugLogsOn
            ),
            title: "settings.privacyAndData.enableDebugLogs".localizedString,
            systemImageName: "ladybug",
            action: {}
        )
    }

    func sentryViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .toggle(
                isOn: $appSettings.isErrorReportingOn
            ),
            title: "settings.anonymousErrorReports.title".localizedString,
            titleTextStyle: .Body.Medium.regular,
            attributtedSubtitle: sentryAttributtedString(),
            imageName: "errorReport",
            action: {}
        )
    }

    func sentryAttributtedString() -> AttributedString {
        let first = AttributedString("settings.anonymousErrorReports.subtitle".localizedString)
        var second = AttributedString("welcome.sentry".localizedString)
        second.underlineStyle = .single
        second.foregroundColor = NymColor.accent
        second.link = URL(string: Constants.sentryURL.rawValue)
        return first + AttributedString(" ") + second + AttributedString(")")
    }

    func statisticsViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .toggle(
                isOn: $appSettings.isStatisticsEnabled
            ),
            title: "welcome.analytics".localizedString,
            titleTextStyle: .Body.Medium.regular,
            subtitle: "welcome.analytics.subtitle".localizedString,
            imageName: "statistics",
            action: {}
        )
    }
}

extension TechnicalOptInsView {
    func continueTapped() {
        ImpactGenerator.shared.impact()
        appSettings.welcomeScreenDidDisplay = true
        path = .init()
    }
}
