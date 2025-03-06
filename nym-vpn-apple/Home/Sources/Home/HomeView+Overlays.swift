import SwiftUI
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
                    configuration: viewModel.offlineOverlayConfiguration
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
                    configuration: viewModel.updateAvailableOverlayConfiguration
                )
            )
            .transition(.opacity)
            .animation(.easeInOut, value: viewModel.isUpdateAvailableOverlayDisplayed)
        }
    }
}
