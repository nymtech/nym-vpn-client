import SwiftUI
import AppSettings
import Device
import Theme
import TunnelStatus
import UIComponents

public struct HomeView: View {
    @StateObject var viewModel: HomeViewModel

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
                gatewaySection()
                connectButton()
            }
            .frame(maxWidth: Device.type == .ipad ? 358 : 390)
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
        .overlay {
            updateAvailableOverlay()
        }
        .snackbar(
            isDisplayed: $viewModel.isSnackBarDisplayed,
            style: .info,
            message: viewModel.systemMessageManager.currentMessage
        )
        .onAppear {
            Task {
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
        StatusAreaView(statusButtonConfig: $viewModel.statusButtonConfig, statusInfoState: $viewModel.statusInfoState)
            .padding(.horizontal, 16)
    }

    @ViewBuilder
    func connectedNoiseAnimation() -> some View {
        if viewModel.lastTunnelStatus == .connected {
            LoopAnimationView(animationName: "connected")
        }
    }

    @ViewBuilder
    func networkModeSection() -> some View {
        HStack {
            Text(viewModel.networkSelectLocalizedTitle)
                .textStyle(.TitleLegacy.Medium.primary)
            Spacer()
            GenericImage(systemImageName: "info.circle", allowsHover: true)
                .foregroundColor(NymColor.sysOutline)
                .frame(width: 14, height: 14)
                .onTapGesture {
                    withAnimation {
                        viewModel.isModeInfoOverlayDisplayed.toggle()
                    }
                }
        }
        .padding(.horizontal, 16)
        Spacer()
            .frame(height: 12)

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
        .padding(.horizontal, 16)
        .onTapGesture {
            viewModel.connectionManager.connectionType = .mixnet5hop
        }
        Spacer()
            .frame(height: 20)
    }

    @ViewBuilder
    func gatewaySection() -> some View {
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
            .frame(height: 20)
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
            .frame(width: 390)
            .onTapGesture {
                viewModel.connectDisconnect()
            }
        Spacer()
            .frame(height: viewModel.appSettings.isSmallScreen || Device.isMacOS ? 24 : 8)
    }
}
