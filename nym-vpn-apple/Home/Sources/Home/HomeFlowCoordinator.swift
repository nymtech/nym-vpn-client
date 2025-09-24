import SwiftUI
#if os(macOS)
import HelperInstall
#endif
import Settings
import UIComponents

struct HomeFlowCoordinator<Content: View>: View {
    @StateObject var state: HomeFlowState

    let content: () -> Content

    var body: some View {
        NavigationStack(path: $state.path) {
            ZStack {
                content()
            }
            .navigationDestination(for: HomeLink.self, destination: linkDestination)
        }
    }
}

private extension HomeFlowCoordinator {
    @ViewBuilder
    private func linkDestination(link: HomeLink) -> some View {
        switch link {
        case .entryGateways:
            GatewaysView(viewModel: GatewaysViewModel(type: .entry, path: $state.path))
        case .exitGateways:
            GatewaysView(viewModel: GatewaysViewModel(type: .exit, path: $state.path))
        case .settings:
            SettingsView(viewModel: SettingsViewModel(path: $state.path))
        case let .gatewayDetails(gateway: gateway, hopType: hopType):
            GatewayDetailsView(path: $state.path, gateway: gateway, hopType: hopType)
#if os(macOS)
        case let .installHelper(afterInstallAction):
            HelperInstallView(
                viewModel: HelperInstallViewModel(
                    path: $state.path,
                    afterInstallAction: afterInstallAction
                )
            )
#endif
        }
    }
}
