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
                    Spacer()
                        .frame(height: NymSpacing.section)
                    settingsList()
                    Spacer()
                        .frame(height: NymSpacing.section)
                    appVersionText()
                        .onTapGesture(count: 3) {
                            viewModel.navigateToSantasMenu()
                        }
                }
                .padding(.horizontal, NymSpacing.large)
            }
            .scrollIndicators(.never)
            .frame(maxWidth: MagicNumbers.maxWidth)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            Color.Nym.background
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
            }
        }
    }

    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.settingsTitle,
            backgroundColorOverride: Color.Nym.background,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() }),
            rightButton: CustomNavBarButton(type: .close, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func settingsList() -> some View {
        SettingsList(viewModel: SettingsListViewModel(sections: viewModel.sections))
    }

    func appVersionText() -> some View {
        VStack(spacing: 0) {
            HStack {
                Text(viewModel.versionTitle)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)
                    .padding(.bottom, NymSpacing.large)
                Spacer()
            }
            Spacer()
                .frame(height: NymSpacing.section)
        }
    }
}
