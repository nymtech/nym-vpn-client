import SwiftUI
import AppSettings
import Modifiers
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
        VStack {
            navbar()
            ScrollView {
                addCredentialsButton()

                Spacer()
                    .frame(height: 24)
                settingsList()
            }
            Spacer()
        }
        .appearanceUpdate()
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }

    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.settingsTitle,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateHome() })
        )
    }

    @ViewBuilder
    func addCredentialsButton() -> some View {
        if viewModel.shouldShowAddCredentials {
            GenericButton(title: "settings.addCredential".localizedString)
                .frame(height: 64)
                .padding(EdgeInsets(top: 24, leading: 16, bottom: 0, trailing: 16))
                .onTapGesture {
                    viewModel.navigateToAddCredentials()
                }
        }
    }

    @ViewBuilder
    func settingsList() -> some View {
        SettingsList(
            viewModel:
                SettingsListViewModel(
                    sections: viewModel.sections,
                    appVersion: viewModel.appVersion()
                )
        )
    }
}
