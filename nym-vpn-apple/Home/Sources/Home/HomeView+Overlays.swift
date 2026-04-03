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
        }
#endif
    }

}
