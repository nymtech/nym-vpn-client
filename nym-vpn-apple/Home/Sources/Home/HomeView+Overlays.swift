import SwiftUI
import Device
import UIComponents

extension HomeView {
    @ViewBuilder
    func modeInfoOverlay() -> some View {
        if viewModel.isModeInfoOverlayDisplayed {
            ModeSelectionInfoView(
                viewModel:
                    ModeSelectionInfoViewModel(
                        externalLinkManager: viewModel.externalLinkManager,
                        isDisplayed: $viewModel.isModeInfoOverlayDisplayed
                    )
            )
            .transition(.opacity)
            .animation(.easeInOut, value: viewModel.isModeInfoOverlayDisplayed)
        }
    }

    @ViewBuilder
    func offlineOverlay() -> some View {
        if viewModel.isOfflineOverlayDisplayed {
            ActionDialogView(
                viewModel: ActionDialogViewModel(
                    isDisplayed: $viewModel.isOfflineOverlayDisplayed,
                    configuration: viewModel.offlineOverlayConfiguration,
                    impactGenerator: .shared
                )
            )
            .transition(.opacity)
            .animation(.easeInOut, value: viewModel.isOfflineOverlayDisplayed)
        }
    }

    @ViewBuilder
    func updateAvailableOverlay() -> some View {
        if viewModel.isUpdateAvailableOverlayDisplayed {
            ActionDialogView(
                viewModel: ActionDialogViewModel(
                    isDisplayed: $viewModel.isUpdateAvailableOverlayDisplayed,
                    configuration: viewModel.updateAvailableOverlayConfiguration,
                    impactGenerator: .shared
                )
            )
            .transition(.opacity)
            .animation(.easeInOut, value: viewModel.isUpdateAvailableOverlayDisplayed)
        }
    }

    @ViewBuilder
    func statisticsEnableOverlay() -> some View {
// TODO: statistics
#if os(macOS)
        if viewModel.isStatisticsOverlayDisplayed,
           !viewModel.appSettings.isStatisticsEnabled,
           Device.isMacOS {
            StatisticsEnableOverlay(
                isPresented: $viewModel.isStatisticsOverlayDisplayed
            )
            .transition(.opacity)
            .animation(.easeInOut, value: viewModel.isStatisticsOverlayDisplayed)
        }
#endif
    }

    @ViewBuilder
    func passphraseOverlay() -> some View {
        if !Device.isMacOS,
           viewModel.appSettings.isSmallScreen,
           viewModel.connectionManager.currentTunnelStatus == .connected,
           !viewModel.appSettings.isPassphraseStored {
            PassphraseOverlay(path: $viewModel.path)
        }
    }
}
