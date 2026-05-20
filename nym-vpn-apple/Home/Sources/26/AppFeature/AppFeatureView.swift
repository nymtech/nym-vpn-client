import SwiftUI
#if os(iOS)
import KeyboardManager
#endif
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
    @State private var drawerHeight: CGFloat = 0
    @State private var welcomeHeight: CGFloat = 0
    @Environment(\.colorScheme)
    private var colorScheme
    @Environment(\.scenePhase)
    private var scenePhase
    @AppStorage(AppSettingKey.currentAppearance.rawValue)
    private var appearance: AppSetting.Appearance = .automatic
    @AppStorage(AppSettingKey.credenitalExists.rawValue)
    private var isCredentialImported = false

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
            GeometryReader { geometry in
                VStack(spacing: 0) {
                    navigationBar
                    ZStack {
                        background
                        GeometryReader { innerGeometry in
                            let effectiveDrawerHeight = viewModel.drawerContent == nil ? 0 : drawerHeight
                            ConnectionStatusBackdrop(viewModel: viewModel.connectionStatus)
                                .position(
                                    x: innerGeometry.size.width / 2,
                                    y: max(0, (innerGeometry.size.height - effectiveDrawerHeight) / 2)
                                )
                        }
                    }
                    .clipped()
                }
                .overlay(alignment: .bottom) {
#if os(iOS)
                    KeyboardHostView(bottomSafeAreaInset: geometry.safeAreaInsets.bottom) {
                        drawer
                            .trackHeight { drawerHeight = $0 }
                    }
#else
                    drawer
                        .trackHeight { drawerHeight = $0 }
#endif
                }
            }
            .ignoresSafeArea(.keyboard, edges: .bottom)
            .animation(.spring, value: viewModel.drawerContent == nil)
            .navigationDestination(for: HomeLink.self, destination: linkDestination)
#if os(iOS)
            .toolbar(.hidden, for: .navigationBar)
#endif
        }
        .nymSnackbar(manager: viewModel.snackbarManager)
        .preferredColorScheme(appearance.colorScheme)
        .onAppear { wireOneClickNavigation() }
        .onChange(of: isCredentialImported) { _, newValue in
            viewModel.handleCredentialChange(imported: newValue)
        }
        .onChange(of: scenePhase) { _, newPhase in
            if newPhase == .active {
                viewModel.handleSceneBecameActive()
            }
        }
        .onChange(of: viewModel.connectionStatus.status) { oldValue, newValue in
            viewModel.handleTunnelStatusChange(from: oldValue, to: newValue)
        }
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

    @ViewBuilder var drawer: some View {
        if viewModel.drawerContent != nil {
            DrawerView(
                tag: viewModel.drawerSlideID,
                onTransitionCompleted: { viewModel.drawerTransitionCompleted() },
                content: drawerContent
            )
            .ignoresSafeArea(.container, edges: .bottom)
            .transition(.move(edge: .bottom))
        }
    }

    @ViewBuilder
    func drawerContent() -> some View {
        ZStack(alignment: .top) {
            switch viewModel.drawerTag {
            case .technicalOptIns:
                WelcomeOptInsView(
                    onContinue: { viewModel.technicalOptInsContinueTapped() }
                )
                .transition(.slideFade(from: .trailing))
            case .welcome:
                welcomeContent
            case .processing:
                if let processingViewModel = viewModel.processingViewModel {
                    ProcessingAccountView(
                        viewModel: processingViewModel,
                        minHeight: welcomeHeight
                    )
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
            }
        }
        .animation(.easeInOut, value: viewModel.drawerTag)
    }

    var welcomeContent: some View {
        AuthFlowView(
            credentialsManager: viewModel.credentialsManager,
            onWillRegister: { flow in viewModel.pendingProcessingFlow = flow }
        )
        .trackHeight { welcomeHeight = $0 }
        .transition(.slideFade(from: .trailing))
    }

    var navigationBar: some View {
        HStack(alignment: .center) {
            ImageButton(
                systemImageName: colorScheme == .light ? "sun.max" : "moon.circle",
                imageSize: Constants.NavigationBar.LeadingIcon.size,
                accessibilityLabel: "home.navigationBar.theme.accessibilityLabel".localizedString
            ) {
                impactGenerator.softImpact()
                viewModel.leadingButtonTapped()
            }
            .padding(.leading, NymSpacing.small)
            Spacer()
            ImageButton(
                systemImageName: "gear",
                imageSize: Constants.NavigationBar.TrailingIcon.size,
                accessibilityLabel: "home.navigationBar.settings.accessibilityLabel".localizedString
            ) {
                impactGenerator.softImpact()
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
        .background((colorScheme == .light ? Color.Nym.backgroundCard : Color.Nym.background).ignoresSafeArea(edges: .top))
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

    }
}
