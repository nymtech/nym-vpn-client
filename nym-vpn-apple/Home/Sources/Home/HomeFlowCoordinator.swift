import AppSettings
import ConfigurationManager
import ConnectionManager
import CredentialsManager
import ExternalLinkManager
import FeatureFlagsManager
import GatewayManager
import ImpactGenerator
import SwiftUI
#if os(iOS)
import PurchasesManager
#elseif os(macOS)
import GRPCManager
#endif
import Routes
import Settings
import UIComponents

struct HomeFlowCoordinator<Content: View>: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var configurationManager: ConfigurationManager
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @EnvironmentObject private var externalLinkManager: ExternalLinkManager
    @EnvironmentObject private var featureFlagsManager: FeatureFlagsManager
    @EnvironmentObject private var gatewayManager: GatewayManager
    @EnvironmentObject private var impactGenerator: ImpactGenerator
#if os(iOS)
    @EnvironmentObject private var purchasesManager: PurchasesManager
#elseif os(macOS)
    @EnvironmentObject private var grpcManager: GRPCManager
#endif
    @State var state: HomeFlowState

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
                    appSettings: appSettings,
                    connectionManager: connectionManager,
                    gatewayManager: gatewayManager,
                    featureFlagsManager: featureFlagsManager
                )
            )
        case .exitGateways:
            GatewaysView(
                viewModel: GatewaysViewModel(
                    type: .exit,
                    path: $state.path,
                    appSettings: appSettings,
                    connectionManager: connectionManager,
                    gatewayManager: gatewayManager,
                    featureFlagsManager: featureFlagsManager
                )
            )
        case .settings:
#if os(iOS)
            SettingsView(
                viewModel:
                    SettingsViewModel(
                        path: $state.path,
                        appSettings: appSettings,
                        configurationManager: configurationManager,
                        connectionManager: connectionManager,
                        credentialsManager: credentialsManager,
                        externalLinkManager: externalLinkManager,
                        featureFlagsManager: featureFlagsManager,
                        impactGenerator: impactGenerator,
                        purchasesManager: purchasesManager
                    )
            )
#elseif os(macOS)
            SettingsView(
                viewModel:
                    SettingsViewModel(
                        isServing: $grpcManager.isServing,
                        path: $state.path,
                        appSettings: appSettings,
                        configurationManager: configurationManager,
                        connectionManager: connectionManager,
                        credentialsManager: credentialsManager,
                        externalLinkManager: externalLinkManager,
                        featureFlagsManager: featureFlagsManager,
                        impactGenerator: impactGenerator
                    )
            )
#endif
        case let .gatewayDetails(gateway: gateway, hopType: hopType):
            ServerDetailsView(path: $state.path, gateway: gateway, hopType: hopType, externalLinkManager: externalLinkManager)
        case .launchView:
            LaunchView(splashScreenDidDisplay: $state.splashScreenDidDisplay, path: $state.path)
        case .onboarding:
            OnboardingView(path: $state.path)
        case .technicalOptIns:
            TechnicalOptInsView(path: $state.path)
        }
    }
}
