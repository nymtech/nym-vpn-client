import SwiftUI
import AppSettings
import ConnectionTypes
import CredentialsManager
import Device
import ConfigurationManager
import UIComponents
import Theme

public struct SettingsView: View {
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @StateObject private var viewModel: SettingsViewModel
#if os(macOS)
    @State private var autologinState = AutologinState()
#endif

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
                VStack(spacing: 0) {
                    credentialOrAddCredentialView()
                    renewButton()

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
            }
            .scrollIndicators(.never)
            .frame(maxWidth: MagicNumbers.maxWidth)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
#if os(macOS)
        .autologinOverlay(
            state: autologinState,
            onRetry: { autologinState.start(kind: .autologinRenew, using: credentialsManager) }
        )
#endif
        .onAppear {
#if os(macOS)
            viewModel.autologinState = autologinState
#endif
            Task {
                await credentialsManager.updateAccountSummary()
                viewModel.reloadSections()
            }
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
    func renewButton() -> some View {
        if viewModel.shouldShowRenewButton {
            GenericButton(title: viewModel.renewButtonTitle)
                .padding(.top, 24)
                .onTapGesture {
                    viewModel.navigateToPlanPurchase()
                }
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
