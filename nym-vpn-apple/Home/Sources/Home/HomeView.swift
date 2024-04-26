import SwiftUI
import Modifiers
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
            isSmallScreen: viewModel.isSmallScreen(),
            content: content
        )
    }
}

private extension HomeView {
    @ViewBuilder
    func content() -> some View {
        VStack {
            navbar()
            Spacer()
            statusAreaSection()
            Spacer()
            networkSection()
            hopSection()
            connectButton()
        }
        .appearanceUpdate()
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .onAppear {
            viewModel.configureConnectedTimeTimer()
        }
        .onDisappear {
            viewModel.stopConnectedTimeTimerUpdates()
        }
    }

    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            rightButton: CustomNavBarButton(type: .settings, action: { viewModel.navigateToSettings() }),
            isSmallScreen: viewModel.isSmallScreen()
        )
    }

    @ViewBuilder
    func statusAreaSection() -> some View {
        StatusButton(
            config: viewModel.statusButtonConfig,
            isSmallScreen: viewModel.isSmallScreen()
        )
        Spacer()
            .frame(height: 8)

        StatusInfoView(
            timeConnected: $viewModel.timeConnected,
            infoState: $viewModel.statusInfoState,
            isSmallScreen: viewModel.isSmallScreen()
        )
    }

    @ViewBuilder
    func networkSection() -> some View {
        HStack {
            Text(viewModel.networkSelectLocalizedTitle)
                .textStyle(.Title.Medium.primary)
            Spacer()
        }
        .padding(.horizontal, 16)
        Spacer()
            .frame(height: viewModel.isSmallScreen() ? 12 : 24)

        NetworkButton(
            viewModel: NetworkButtonViewModel(
                type: .mixnet5hop,
                selectedNetwork: $viewModel.selectedNetwork,
                isSmallScreen: viewModel.isSmallScreen()
            )
        )
        .padding(EdgeInsets(top: 0, leading: 16, bottom: 12, trailing: 16))
        .onTapGesture {
            viewModel.selectedNetwork = .mixnet5hop
        }

        NetworkButton(
            viewModel: NetworkButtonViewModel(
                type: .mixnet2hop,
                selectedNetwork: $viewModel.selectedNetwork,
                isSmallScreen: viewModel.isSmallScreen()
            )
        )
        .padding(.horizontal, 16)
        .onTapGesture {
            viewModel.selectedNetwork = .mixnet2hop
        }
        Spacer()
            .frame(height: viewModel.isSmallScreen() ? 20 : 32)
    }

    @ViewBuilder
    func hopSection() -> some View {
        HStack {
            Text(viewModel.connectToLocalizedTitle)
                .foregroundStyle(NymColor.sysOnSurfaceWhite)
                .textStyle(.Title.Medium.primary)
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
            .frame(height: viewModel.isSmallScreen() ? 20 : 32)
    }

    @ViewBuilder func entryHop() -> some View {
        if viewModel.shouldShowEntryHop() {
            HopButton(viewModel: viewModel.entryHopButtonViewModel)
                .animation(.default, value: viewModel.connectionManager.entryGateway)
                .onTapGesture {
                    viewModel.navigateToFirstHopSelection()
                }
            Spacer()
                .frame(height: 20)
        }
    }

    @ViewBuilder func exitHop() -> some View {
        HopButton(viewModel: viewModel.exitHopButtonViewModel)
            .animation(.default, value: viewModel.connectionManager.exitRouter)
            .onTapGesture {
                viewModel.navigateToLastHopSelection()
            }
    }

    @ViewBuilder
    func connectButton() -> some View {
        ConnectButton(state: viewModel.connectButtonState)
            .padding(.horizontal, 16)
            .onTapGesture {
                viewModel.connectDisconnect()
            }
        if viewModel.isSmallScreen() {
            Spacer()
                .frame(height: 24)
        }
    }
}
