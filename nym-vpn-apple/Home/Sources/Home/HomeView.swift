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
            .frame(maxWidth: MagicNumbers.maxWidth)
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
        .overlay {
            statisticsEnableOverlay()
        }
        .snackbar(
            isDisplayed: $viewModel.isSnackBarDisplayed,
            message: viewModel.messagesManager.currentMessage
        )
        .onAppear {
            Task {
                try? await Task.sleep(for: .seconds(3))
                viewModel.messagesManager.processMessages()
            }
        }
    }

    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            useElevationBackground: true,
            rightButton: CustomNavBarButton(type: .settings, action: { viewModel.navigateToSettings() })
        )
    }

    func statusAreaSection() -> some View {
        StatusAreaView(
            statusButtonConfig: $viewModel.statusButtonConfig,
            statusInfoState: $viewModel.statusInfoState,
            connectedDate: $viewModel.connectionManager.connectedDate
        )
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
                .textStyle(.Headline.Small.regular)
            Spacer()
            GenericImage(systemImageName: "info.circle", allowsHover: true)
                .foregroundColor(NymColor.gray1)
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

        NetworkButton(viewModel: viewModel.fastButtonViewModel)
            .padding(EdgeInsets(top: 0, leading: 16, bottom: 12, trailing: 16))
            .onTapGesture {
                viewModel.connectionManager.connectionType = .wireguard
            }
            .accessibilityAction {
                viewModel.connectionManager.connectionType = .wireguard
            }

        NetworkButton(viewModel: viewModel.anonymousButtonViewModel)
            .padding(.horizontal, 16)
            .onTapGesture {
                viewModel.connectionManager.connectionType = .mixnet5hop
            }
            .accessibilityAction {
                viewModel.connectionManager.connectionType = .mixnet5hop
            }
        Spacer()
            .frame(height: 20)
    }

    @ViewBuilder
    func gatewaySection() -> some View {
        HStack {
            Text(viewModel.connectToLocalizedTitle)
                .foregroundStyle(NymColor.primary)
                .textStyle(.Headline.Small.regular)
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
        HopButton(hopType: .entry)
            .animation(.default, value: viewModel.connectionManager.entryGateway)
            .onTapGesture {
                viewModel.navigateToEntryGateways()
            }
            .accessibilityAction {
                viewModel.navigateToEntryGateways()
            }
        Spacer()
            .frame(height: 20)
    }

    @ViewBuilder
    func exitHop() -> some View {
        HopButton(hopType: .exit)
            .animation(.default, value: viewModel.connectionManager.exitRouter)
            .onTapGesture {
                viewModel.navigateToExitGateways()
            }
            .accessibilityAction {
                viewModel.navigateToExitGateways()
            }
    }

    @ViewBuilder
    func connectButton() -> some View {
        ConnectButton(state: viewModel.connectButtonState)
            .padding(.horizontal, 16)
            .frame(maxWidth: MagicNumbers.maxWidth)
            .onTapGesture {
                Task { @MainActor in
                    await viewModel.connectDisconnect()
                }
            }
            .accessibilityAction {
                Task { @MainActor in
                    await viewModel.connectDisconnect()
                }
            }
        Spacer()
            .frame(height: viewModel.appSettings.isSmallScreen || Device.isMacOS ? 24 : 8)
    }
}
