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
            GatewaysView(
                viewModel: GatewaysViewModel(
                    type: .entry,
                    path: $state.path,
                    appSettings: .shared,
                    connectionManager: .shared,
                    gatewayManager: .shared,
                    featureFlagsManager: .shared
                )
            )
        case .exitGateways:
            GatewaysView(
                viewModel: GatewaysViewModel(
                    type: .exit,
                    path: $state.path,
                    appSettings: .shared,
                    connectionManager: .shared,
                    gatewayManager: .shared,
                    featureFlagsManager: .shared
                )
            )
        case .settings:
#if os(iOS)
            SettingsView(
                viewModel:
                    SettingsViewModel(
                        path: $state.path,
                        appSettings: .shared,
                        configurationManager: .shared,
                        connectionManager: .shared,
                        credentialsManager: .shared,
                        externalLinkManager: .shared,
                        featureFlagsManager: .shared
                    )
            )
#elseif os(macOS)
            SettingsView(
                viewModel:
                    SettingsViewModel(
                        path: $state.path,
                        appSettings: .shared,
                        configurationManager: .shared,
                        connectionManager: .shared,
                        credentialsManager: .shared,
                        externalLinkManager: .shared,
                        helperManager: .shared,
                        featureFlagsManager: .shared
                    )
            )
#endif
        case let .gatewayDetails(gateway: gateway, hopType: hopType):
            GatewayDetailsView(path: $state.path, gateway: gateway, hopType: hopType, externalLinkManager: .shared)
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
