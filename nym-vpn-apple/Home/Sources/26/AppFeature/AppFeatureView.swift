import SwiftUI
import AppSettings
import ConfigurationManager
import ConnectionManager
import CredentialsManager
import ExternalLinkManager
import FeatureFlagsManager
import GatewayManager
#if os(macOS)
import GRPCManager
#endif
import ImpactGenerator
#if os(iOS)
import PurchasesManager
#endif
import Routes
import Settings
import Theme
import UIComponents

public struct AppFeatureView: View {
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

    @State private var viewModel: AppFeatureViewModel
    @State private var welcomeHeight: CGFloat = 0
    @State private var drawerHeight: CGFloat = 0
    @Environment(\.colorScheme)
    private var colorScheme
    @AppStorage(AppSettingKey.currentAppearance.rawValue)
    private var appearance: AppSetting.Appearance = .automatic

    public init(viewModel: AppFeatureViewModel) {
        _viewModel = State(wrappedValue: viewModel)
    }

    private var pathBinding: Binding<NavigationPath> {
        Binding(
            get: { viewModel.path },
            set: { viewModel.path = $0 }
        )
    }

    public var body: some View {
        NavigationStack(path: pathBinding) {
            ZStack {
                background
                connectionStatusBackdrop
                VStack {
                    navigationBar
                    Spacer()
                }
                drawer
            }
            .animation(.spring, value: viewModel.drawerContent == nil)
            .animation(Constants.Backdrop.animation, value: drawerHeight)
            .navigationDestination(for: HomeLink.self, destination: linkDestination)
#if os(iOS)
            .toolbar(.hidden, for: .navigationBar)
#endif
        }
        .nymSnackbar(manager: viewModel.snackbarManager)
        .preferredColorScheme(appearance.colorScheme)
        .onAppear { wireOneClickNavigation() }
    }
}

#if DEBUG
#Preview {
#if os(iOS)
    AppFeatureView(
        viewModel: AppFeatureViewModel(
            appSettings: .shared,
            credentialsManager: .shared,
            connectionManager: .shared,
            gatewayManager: .shared,
            snackbarManager: .shared,
            impactGenerator: .shared,
            networkMonitor: .shared
        )
    )
#elseif os(macOS)
    AppFeatureView(
        viewModel: AppFeatureViewModel(
            appSettings: .shared,
            credentialsManager: .shared,
            connectionManager: .shared,
            gatewayManager: .shared,
            snackbarManager: .shared,
            impactGenerator: .shared,
            networkMonitor: .shared,
            grpcManager: .shared
        )
    )
#endif
}
#endif

private extension AppFeatureView {
    func wireOneClickNavigation() {
        let pushPlanPurchase: () -> Void = { [weak viewModel] in
            guard let viewModel else { return }
            viewModel.path.append(HomeLink.settings)
            viewModel.path.append(SettingLink.generatePassphrase(displayPurchaseView: true))
        }
        viewModel.oneClick.onRequestPlanPurchase = pushPlanPurchase
        viewModel.onRequestPlanPurchase = pushPlanPurchase
#if os(macOS)
        viewModel.oneClick.onRequestDaemonEnable = { [weak viewModel] in
            guard let viewModel else { return }
            viewModel.path.append(HomeLink.settings)
            viewModel.path.append(SettingLink.daemonEnable)
        }
#endif
    }

    var background: some View {
        Color.Nym.background
            .ignoresSafeArea()
    }

    var connectionStatusBackdrop: some View {
        VStack(spacing: 0) {
            Color.clear
                .frame(height: Constants.NavigationBar.totalHeight)
                .accessibilityHidden(true)
            Spacer(minLength: 0)
            ConnectionStatusBackdrop(viewModel: viewModel.connectionStatus)
            Spacer(minLength: 0)
            Color.clear
                .frame(height: drawerFootprint)
                .accessibilityHidden(true)
        }
    }

    var drawerFootprint: CGFloat {
        guard drawerHeight > 0 else { return 0 }
        return drawerHeight + NymSpacing.large
    }

    @ViewBuilder var drawer: some View {
        if viewModel.drawerContent != nil {
            DrawerView(
                tag: viewModel.drawerSlideID,
                onTransitionCompleted: { viewModel.drawerTransitionCompleted() },
                content: drawerContent
            )
            .padding(.top, Constants.NavigationBar.totalHeight)
            .ignoresSafeArea(.container, edges: .bottom)
            .transition(.move(edge: .bottom))
        }
    }

    @ViewBuilder
    func drawerContent() -> some View {
        ZStack(alignment: .top) {
            switch viewModel.drawerTag {
            case .welcome:
                AuthFlowView(credentialsManager: viewModel.credentialsManager)
                    .trackHeight { newHeight in
                        welcomeHeight = newHeight
                        drawerHeight = newHeight
                    }
                    .transition(.slideFade(from: .trailing))
            case .processing:
                if let processingViewModel = viewModel.processingViewModel {
                    ProcessingAccountView(
                        viewModel: processingViewModel,
                        minHeight: welcomeHeight
                    )
                    .trackHeight { drawerHeight = $0 }
                    .transition(.slideFade(from: .trailing))
                } else {
                    Color.clear.frame(height: 1)
                }
            case .oneClick:
                OneClickView(
                    viewModel: viewModel.oneClick,
                    onSelectEntry: { viewModel.path.append(HomeLink.entryGateways) },
                    onSelectExit: { viewModel.path.append(HomeLink.exitGateways) }
                )
                .trackHeight { drawerHeight = $0 }
            }
        }
        .animation(.easeInOut, value: viewModel.drawerTag)
    }

    var navigationBar: some View {
        HStack(alignment: .center) {
            ImageButton(
                systemImageName: colorScheme == .light ? "sun.max" : "moon.circle",
                imageSize: Constants.NavigationBar.LeadingIcon.size,
                accessibilityLabel: "home.navigationBar.theme.accessibilityLabel".localizedString
            ) {
                viewModel.leadingButtonTapped()
            }
            .padding(.leading, NymSpacing.small)
            Spacer()
            ImageButton(
                systemImageName: "gear",
                imageSize: Constants.NavigationBar.TrailingIcon.size,
                accessibilityLabel: "home.navigationBar.settings.accessibilityLabel".localizedString
            ) {
                viewModel.path.append(HomeLink.settings)
            }
            .padding(.leading, NymSpacing.small)
        }
        .frame(height: Constants.NavigationBar.height)
        .padding(.horizontal, NymSpacing.component)
        .padding(.vertical, NymSpacing.small)
        .overlay {
            if viewModel.shouldShowLogo {
                GenericImage(imageName: "logoText")
                    .frame(width: Constants.NavigationBar.Logo.width)
                    .allowsHitTesting(false)
                    .transition(.move(edge: .top).combined(with: .opacity))
            }
        }
        .clipped()
        .background((colorScheme == .light ? Color.white : Color.Nym.background).ignoresSafeArea(edges: .top))
        .animation(.easeInOut(duration: 0.35), value: viewModel.shouldShowLogo)
    }
}

private extension AppFeatureView {
    @ViewBuilder
    func linkDestination(link: HomeLink) -> some View {
        switch link {
        case .settings:
            settingsDestination()
        case .entryGateways:
            GatewaysView(
                viewModel: GatewaysViewModel(
                    type: .entry,
                    path: pathBinding,
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
                    path: pathBinding,
                    appSettings: appSettings,
                    connectionManager: connectionManager,
                    gatewayManager: gatewayManager,
                    featureFlagsManager: featureFlagsManager
                )
            )
        case let .gatewayDetails(gateway: gateway, hopType: hopType):
            ServerDetailsView(
                path: pathBinding,
                gateway: gateway,
                hopType: hopType,
                externalLinkManager: externalLinkManager
            )
        default:
            EmptyView()
        }
    }

    @ViewBuilder
    func settingsDestination() -> some View {
#if os(iOS)
        SettingsView(
            viewModel: SettingsViewModel(
                path: pathBinding,
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
            viewModel: SettingsViewModel(
                isServing: $grpcManager.isServing,
                path: pathBinding,
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
    }
}

private extension AppFeatureView {
    enum Constants {
        enum NavigationBar {
            static let height: CGFloat = 64
            static let totalHeight: CGFloat = height + NymSpacing.small * 2

            enum LeadingIcon {
                static let size: CGFloat = 24
            }

            enum Logo {
                static let width: CGFloat = 120
            }

            enum TrailingIcon {
                static let size: CGFloat = 24
            }
        }

        enum Backdrop {
            static let animation = SwiftUI.Animation.spring(response: 0.35, dampingFraction: 0.8)
        }
    }
}
