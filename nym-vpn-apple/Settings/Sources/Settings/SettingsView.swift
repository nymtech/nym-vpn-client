import SwiftUI
import AppSettings
import Device
import ConfigurationManager
import UIComponents
import Theme

public struct SettingsView: View {
    @StateObject private var viewModel: SettingsViewModel

    public init(viewModel: SettingsViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    public var body: some View {
        SettingsFlowCoordinator(flowState: viewModel, content: content)
    }
}

private extension SettingsView {
    @ViewBuilder
    func content() -> some View {
        VStack(spacing: 0) {
            navbar()
            ScrollView {
                credentialOrAddCredentialView()

                Spacer()
                    .frame(height: 24)
                settingsList()
                Spacer()
                    .frame(height: 24)
                appVersionText()
                    .onTapGesture(count: 3) {
                        viewModel.navigateToSantasMenu()
                    }
            }
            .padding(.horizontal, 16)
            .scrollIndicators(.hidden)
            .frame(maxWidth: MagicNumbers.maxWidth)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .overlay {
            if viewModel.isLogoutConfirmationDisplayed {
                ActionDialogView(
                    viewModel: ActionDialogViewModel(
                        isDisplayed: $viewModel.isLogoutConfirmationDisplayed,
                        configuration: viewModel.logoutDialogConfiguration,
                        impactGenerator: .shared,
                        isLoading: $viewModel.isLogoutLoading
                    )
                )
            }
        }
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }

    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.settingsTitle,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func credentialOrAddCredentialView() -> some View {
        if !viewModel.isValidCredentialImported {
            loginButton()
        }
    }

    @ViewBuilder
    func loginButton() -> some View {
        GenericButton(title: "settings.getStarted".localizedString)
            .frame(height: 64)
            .padding(EdgeInsets(top: 24, leading: 0, bottom: 0, trailing: 0))
            .onTapGesture {
                viewModel.navigateToOnboardingOrCredential()
            }
    }

    @ViewBuilder
    func settingsList() -> some View {
        SettingsList(viewModel: SettingsListViewModel(sections: viewModel.sections))
    }

    func appVersionText() -> some View {
        VStack(spacing: 0) {
            HStack {
                Text(viewModel.versionTitle)
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Medium.regular)
                    .padding(.bottom, 16)
                Spacer()
            }
            Spacer()
                .frame(height: 24)
        }
    }
}
