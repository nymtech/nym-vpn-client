import SwiftUI
import AppSettings
import Modifiers
import Theme
import UIComponents

public struct HomeView: View {
    @ObservedObject private var viewModel: HomeViewModel

    public init(viewModel: HomeViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        HomeFlowCoordinator(state: viewModel, isSmallScreen: viewModel.isSmallScreen(), content: content)
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
            connectionSection()
            connectButton()
        }
        .appearanceUpdate()
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }

    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: "NymVPN".localizedString,
            rightButton: CustomNavBarButton(type: .settings, action: { viewModel.navigateToSettings() }),
            isSmallScreen: viewModel.isSmallScreen()
        )
    }

    @ViewBuilder
    func statusAreaSection() -> some View {
        StatusButton(config: .disconnected, isSmallScreen: viewModel.isSmallScreen())
        Spacer()
            .frame(height: 8)

        StatusInfoView(isSmallScreen: viewModel.isSmallScreen())
    }

    @ViewBuilder
    func networkSection() -> some View {
        HStack {
            Text("selectNetwork".localizedString)
                .textStyle(.Title.Medium.primary)
            Spacer()
        }
        .padding(.horizontal, 16)
        Spacer()
            .frame(height: viewModel.isSmallScreen() ? 12 : 24)

        NetworkButton(
            viewModel: NetworkButtonViewModel(
                type: .mixnet,
                selectedNetwork: $viewModel.selectedNetwork,
                isSmallScreen: viewModel.isSmallScreen()
            )
        )
        .padding(EdgeInsets(top: 0, leading: 16, bottom: 12, trailing: 16))
        .onTapGesture {
            viewModel.selectedNetwork = .mixnet
        }

        NetworkButton(
            viewModel: NetworkButtonViewModel(
                type: .wireguard,
                selectedNetwork: $viewModel.selectedNetwork,
                isSmallScreen: viewModel.isSmallScreen()
            )
        )
        .padding(.horizontal, 16)
        .onTapGesture {
            viewModel.selectedNetwork = .wireguard
        }
        Spacer()
            .frame(height: viewModel.isSmallScreen() ? 20 : 32)
    }

    @ViewBuilder
    func connectionSection() -> some View {
        HStack {
            Text("connectTo".localizedString)
                .foregroundStyle(NymColor.sysOnSurfaceWhite)
                .textStyle(.Title.Medium.primary)
            Spacer()
        }
        .padding(.horizontal, 16)

        Spacer()
            .frame(height: 20)

        VStack {
            HopButton(hopType: .first, country: Country(name: "Germany", code: "de"))
                .onTapGesture {
                    viewModel.navigateToFirstHopSelection()
                }
            Spacer()
                .frame(height: 20)
            HopButton(hopType: .last, country: Country(name: "Switzerland", code: "ch"))
                .onTapGesture {
                    viewModel.navigateToLastHopSelection()
                }
        }
        .padding(.horizontal, 16)

        Spacer()
            .frame(height: viewModel.isSmallScreen() ? 20 : 32)
    }

    @ViewBuilder
    func connectButton() -> some View {
        ConnectButton()
            .padding(.horizontal, 16)
            .onTapGesture {
                viewModel.connect()
            }
        if viewModel.isSmallScreen() {
            Spacer()
                .frame(height: 24)
        }
    }
}
