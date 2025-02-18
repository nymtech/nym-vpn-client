import SwiftUI
import AppSettings
import Device
import Theme
import TunnelStatus
import UIComponents

public struct HomeView: View {
    @StateObject private var viewModel: HomeViewModel

    public init(viewModel: HomeViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    public var body: some View {
        HomeFlowCoordinator(
            state: viewModel,
            content: content
        )
    }
}

private extension HomeView {
    @ViewBuilder
    func content() -> some View {
        VStack {
            navbar()
            VStack {
                Spacer()
                statusAreaSection()
                Spacer()
                networkModeSection()
                countryHopSection()
                connectButton()
            }
            .frame(maxWidth: Device.type == .ipad ? 358 : .infinity)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .overlay {
            modeInfoOverlay()
        }
        .overlay {
            offlineOverlay()
        }
        .snackbar(
            isDisplayed: $viewModel.isSnackBarDisplayed,
            style: .info,
            message: viewModel.systemMessageManager.currentMessage
        )
        .onAppear {
            Task(priority: .background) {
                try? await Task.sleep(for: .seconds(3))
                viewModel.systemMessageManager.processMessages()
            }
        }
    }

    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            isHomeScreen: true,
            rightButton: CustomNavBarButton(type: .settings, action: { viewModel.navigateToSettings() })
        )
    }

    @ViewBuilder
    func statusAreaSection() -> some View {
        VStack {
            StatusButton(
                config: viewModel.statusButtonConfig,
                isSmallScreen: viewModel.appSettings.isSmallScreen
            )
            Spacer()
                .frame(height: 8)

            StatusInfoView(
                timeConnected: $viewModel.timeConnected,
                infoState: $viewModel.statusInfoState,
                isSmallScreen: viewModel.appSettings.isSmallScreen
            )
        }
        .padding(.horizontal, 16)
    }

    @ViewBuilder
    func networkModeSection() -> some View {
        HStack {
            Text(viewModel.networkSelectLocalizedTitle)
                .textStyle(.TitleLegacy.Medium.primary)
            Spacer()
            Image(systemName: "info.circle")
                .foregroundColor(NymColor.sysOutline)
                .frame(width: 24, height: 24)
                .onTapGesture {
                    withAnimation {
                        viewModel.isModeInfoOverlayDisplayed.toggle()
                    }
                }
        }
        .padding(.horizontal, 16)
        Spacer()
            .frame(height: viewModel.appSettings.isSmallScreen ? 12 : 24)

        NetworkButton(
            viewModel: viewModel.fastButtonViewModel
        )
        .padding(EdgeInsets(top: 0, leading: 16, bottom: 12, trailing: 16))
        .onTapGesture {
            viewModel.connectionManager.connectionType = .wireguard
        }

        NetworkButton(
            viewModel: viewModel.anonymousButtonViewModel
        )
        .opacity(1.0)
        .padding(.horizontal, 16)
        .onTapGesture {
            viewModel.connectionManager.connectionType = .mixnet5hop
        }
        Spacer()
            .frame(height: viewModel.appSettings.isSmallScreen ? 20 : 32)
    }

    @ViewBuilder
    func countryHopSection() -> some View {
        HStack {
            Text(viewModel.connectToLocalizedTitle)
                .foregroundStyle(NymColor.sysOnSurfaceWhite)
                .textStyle(.TitleLegacy.Medium.primary)
            Spacer()
        }
        .padding(.horizontal, 16)

        Spacer()
            .frame(height: 20)

        VStack {
            entryHop()
            exitHop()
        }
        .padding(.horizontal, 16)

        Spacer()
            .frame(height: viewModel.appSettings.isSmallScreen ? 20 : 32)
    }

    @ViewBuilder
    func entryHop() -> some View {
        HopButton(
            viewModel:
                HopButtonViewModel(
                    hopType: .entry,
                    entryGateway: $viewModel.connectionManager.entryGateway,
                    exitRouter: $viewModel.connectionManager.exitRouter
                )
        )
        .animation(.default, value: viewModel.connectionManager.entryGateway)
        .onTapGesture {
            viewModel.navigateToEntryGateways()
        }
        Spacer()
            .frame(height: 20)
    }

    @ViewBuilder
    func exitHop() -> some View {
        HopButton(
            viewModel:
                HopButtonViewModel(
                    hopType: .exit,
                    entryGateway: $viewModel.connectionManager.entryGateway,
                    exitRouter: $viewModel.connectionManager.exitRouter
                )
        )
        .animation(.default, value: viewModel.connectionManager.exitRouter)
        .onTapGesture {
            viewModel.navigateToExitGateways()
        }
    }

    @ViewBuilder
    func connectButton() -> some View {
        ConnectButton(state: viewModel.connectButtonState)
            .padding(.horizontal, 16)
            .onTapGesture {
                viewModel.connectDisconnect()
            }
            Spacer()
            .frame(height: viewModel.appSettings.isSmallScreen || Device.isMacOS ? 24 : 8)
    }
}

// MARK: - Overlays -
private extension HomeView {
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
}
