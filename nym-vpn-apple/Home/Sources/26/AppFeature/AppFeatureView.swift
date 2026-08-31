import SwiftUI
import AccountPrefetchGates
import AppSettings
#if os(iOS)
import KeyboardManager
#endif
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
    @State private var bottomSafeAreaInset: CGFloat = 0
    @State private var drawerOffsetY: CGFloat = 0
#if os(macOS)
    @State private var autologinState = AutologinState()
#endif
    @Environment(\.colorScheme)
    private var colorScheme
    @Environment(\.scenePhase)
    private var scenePhase
    @AppStorage(AppSettingKey.currentAppearance.rawValue)
    private var appearance: AppSetting.Appearance = .automatic
    @AppStorage(AppSettingKey.credenitalExists.rawValue)
    private var isCredentialImported = false
    @AppStorage(AppSettingKey.onboardingDidDisplay.rawValue)
    private var onboardingDidDisplay = false

    public init(viewModel: AppFeatureViewModel) {
        _viewModel = State(wrappedValue: viewModel)
    }

    public var body: some View {
        @Bindable var viewModel = viewModel
        decoratedRoot(viewModel: viewModel)
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

private struct ConnectionStatusBackdropLayer: View {
    let connectionStatus: ConnectionStatusViewModel
    let drawerContentIsNil: Bool
    let drawerHeight: CGFloat
    let bottomSafeAreaInset: CGFloat

    var body: some View {
        GeometryReader { innerGeometry in
            let effectiveDrawerHeight = drawerContentIsNil ? 0 : drawerHeight
            let availableHeight = max(
                0,
                innerGeometry.size.height + bottomSafeAreaInset - effectiveDrawerHeight
            )
            ConnectionStatusBackdrop(
                viewModel: connectionStatus,
                availableHeight: availableHeight
            )
            .position(
                x: innerGeometry.size.width / 2,
                y: availableHeight / 2
            )
            .animation(
                .spring(response: 0.35, dampingFraction: 0.85),
                value: effectiveDrawerHeight
            )
        }
    }
}

private extension AppFeatureView {
    @ViewBuilder
    func decoratedRoot(viewModel: AppFeatureViewModel) -> some View {
        @Bindable var viewModel = viewModel
        let stack = navigationStack(viewModel: viewModel)
        withSessionObservers(stack, viewModel: viewModel)
            .nymSnackbar(manager: viewModel.snackbarManager)
            .overlay {
                if viewModel.isFamilyWarningModalDisplayed {
                    ModalOverlayView(
                        isDisplayed: $viewModel.isFamilyWarningModalDisplayed,
                        dismissOnOverlayTap: false,
                        horizontalPadding: NymSpacing.standard,
                        maxWidth: NymSpacing.drawerMaxWidth
                    ) {
                        FamilyWarningModalView(
                            title: "gatewayIndependence.modal.title".localizedString,
                            reminderText: "gatewayIndependence.modal.disableReminders".localizedString,
                            reminderLinkText: "gatewayIndependence.modal.notificationSettingsLink".localizedString,
                            connectAnywayTitle: "gatewayIndependence.warning.connectAnyway".localizedString,
                            cancelTitle: "cancel".localizedString,
                            onConnectAnyway: { viewModel.confirmFamilyWarning() },
                            onCancel: { viewModel.dismissFamilyWarning() },
                            onOpenNotificationSettings: { viewModel.openNotificationSettingsFromFamilyWarning() }
                        )
                    }
                }
            }
            .overlay {
                if !onboardingDidDisplay, !isCredentialImported {
                    OnboardingView {
                        appSettings.onboardingDidDisplay = true
                    }
                    .transition(.opacity)
                }
            }
            .animation(.easeInOut, value: onboardingDidDisplay)
            .preferredColorScheme(appearance.colorScheme)
            .onAppear { wireMacOSDaemonNavigation() }
#if os(macOS)
            .modifier(
                WebSubscriptionPurchaseChromeModifier(
                    viewModel: viewModel,
                    autologinState: autologinState,
                    credentialsManager: credentialsManager
                )
            )
#endif
    }

    @ViewBuilder
    func navigationStack(viewModel: AppFeatureViewModel) -> some View {
        @Bindable var viewModel = viewModel
        NavigationStack(path: $viewModel.path) {
            homeColumn(viewModel: viewModel)
                .overlay(alignment: .bottom) {
                    drawerOverlay(viewModel: viewModel)
                }
                .background { bottomSafeAreaReader }
                .ignoresSafeArea(.keyboard, edges: .bottom)
                .animation(drawerPresenceAnimation(viewModel: viewModel), value: viewModel.drawerContent == nil)
                .navigationDestination(for: HomeLink.self) { link in
                    linkDestination(link: link, path: $viewModel.path)
                }
#if os(iOS)
                .toolbar(.hidden, for: .navigationBar)
#endif
        }
    }

    @ViewBuilder
    func homeColumn(viewModel: AppFeatureViewModel) -> some View {
        VStack(spacing: 0) {
            navigationBar
            ZStack {
                background
                if !viewModel.purchaseTransitionOverlayVisible {
                    ConnectionStatusBackdropLayer(
                        connectionStatus: viewModel.connectionStatus,
                        drawerContentIsNil: viewModel.drawerContent == nil,
                        drawerHeight: drawerHeight,
                        bottomSafeAreaInset: bottomSafeAreaInset
                    )
                }
                if viewModel.purchaseTransitionOverlayVisible {
                    Color.Nym.background
                        .ignoresSafeArea()
                        .allowsHitTesting(false)
                }
            }
            .clipped()
        }
    }

    @ViewBuilder
    func drawerOverlay(viewModel: AppFeatureViewModel) -> some View {
#if os(iOS)
        KeyboardHostView(
            bottomSafeAreaInset: bottomSafeAreaInset,
            isEnabled: viewModel.drawerTag.isWelcome
        ) {
            Spacer()
            drawerColumn
                .trackHeight { drawerHeight = $0 }
                .padding(.bottom, bottomSafeAreaInset == 0 ? NymSpacing.standard : 0)
        }
#else
        drawerColumn
            .trackHeight { drawerHeight = $0 }
            .padding(.bottom, NymSpacing.standard)
#endif
    }

    var bottomSafeAreaReader: some View {
        GeometryReader { proxy in
            Color.clear
                .onAppear { bottomSafeAreaInset = proxy.safeAreaInsets.bottom }
                .onChange(of: proxy.safeAreaInsets.bottom) { _, newValue in
                    bottomSafeAreaInset = newValue
                }
        }
    }

    @ViewBuilder
    func withSessionObservers<Content: View>(
        _ content: Content,
        viewModel: AppFeatureViewModel
    ) -> some View {
        content
            .onChange(of: viewModel.planPurchaseNavigationToken) { _, _ in
                handlePlanPurchaseNavigationTokenChange(viewModel: viewModel)
            }
            .onChange(of: viewModel.drawerContent == nil) { _, drawerHidden in
                handleDrawerHiddenChange(drawerHidden, viewModel: viewModel)
            }
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

    func handlePlanPurchaseNavigationTokenChange(viewModel: AppFeatureViewModel) {
        guard viewModel.navigationIntent == .pushPlanPurchase else { return }
        guard PurchaseTransitionPolicy.shouldPushPlanPurchaseAfterDrawerHidden(
            drawerHidden: viewModel.drawerContent == nil,
            checkoutNavigationPending: viewModel.isCheckoutNavigationPending
        ) else { return }
        pushPlanPurchaseNavigation()
        viewModel.consumeNavigationIntent()
        viewModel.checkoutNavigationDidComplete()
    }

    func handleDrawerHiddenChange(_ drawerHidden: Bool, viewModel: AppFeatureViewModel) {
        guard viewModel.navigationIntent == .pushPlanPurchase else { return }
        guard PurchaseTransitionPolicy.shouldPushPlanPurchaseAfterDrawerHidden(
            drawerHidden: drawerHidden,
            checkoutNavigationPending: viewModel.isCheckoutNavigationPending
        ) else { return }
        pushPlanPurchaseNavigation()
        viewModel.consumeNavigationIntent()
        viewModel.checkoutNavigationDidComplete()
    }

    func drawerPresenceAnimation(viewModel: AppFeatureViewModel) -> Animation {
        if PurchaseTransitionPolicy.usesTimedDrawerHide(
            isPlanPurchasePending: viewModel.navigationIntent == .pushPlanPurchase
        ) {
            return .easeInOut(duration: PurchaseTransitionPolicy.navigationPushAnimationDurationSeconds)
        }
        return .spring
    }

    func pushPlanPurchaseNavigation() {
        withAnimation(.easeInOut(duration: PurchaseTransitionPolicy.navigationPushAnimationDurationSeconds)) {
            viewModel.path = NavigationPath()
            viewModel.path.append(HomeLink.settings)
            viewModel.path.append(SettingLink.generatePassphrase(displayPurchaseView: true))
        }
    }

    func wireMacOSDaemonNavigation() {
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
            DrawerView(content: drawerContent)
                .ignoresSafeArea(.container, edges: .bottom)
                .transition(.move(edge: .bottom))
        }
    }

    @ViewBuilder var drawerColumn: some View {
        VStack(spacing: 0) {
            if viewModel.drawerTag == .oneClick {
                speedModeSelector
            }
            drawer
        }
        .offset(y: drawerOffsetY)
        .opacity(viewModel.shouldHideDrawerChromeDuringCheckout ? 0 : 1)
        .allowsHitTesting(!viewModel.shouldHideDrawerChromeDuringCheckout)
        .onChange(of: viewModel.drawerSlideID) { _, _ in
            slideDrawer()
        }
    }

    func slideDrawer() {
        withAnimation(.easeIn) {
            drawerOffsetY = DrawerSlide.offset
        } completion: {
            viewModel.drawerTransitionCompleted()
            withAnimation(.spring) {
                drawerOffsetY = 0
            }
        }
    }

    var speedModeSelector: some View {
        SpeedModeSegmentedControl(selection: viewModel.oneClick.speedMode) { mode in
            viewModel.oneClick.setSpeedMode(mode)
        }
        .frame(maxWidth: NymSpacing.drawerMaxWidth)
        .padding(.horizontal, NymSpacing.standard)
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
                    onSelectExit: { viewModel.path.append(HomeLink.exitGateways) },
                    onShowGatewayDetails: { gateway, hopType in
                        viewModel.path.append(
                            HomeLink.gatewayDetails(gateway: gateway, hopType: hopType)
                        )
                    }
                )
            }
        }
        .animation(.easeInOut, value: viewModel.drawerTag)
    }

    var welcomeContent: some View {
        AuthFlowView(
            credentialsManager: viewModel.credentialsManager,
            sessionCoordinator: viewModel
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
                guard viewModel.drawerContent?.isProcessing != true else { return }
                impactGenerator.softImpact()
                viewModel.path.append(HomeLink.settings)
            }
            .allowsHitTesting(viewModel.drawerContent?.isProcessing != true)
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
        .background((colorScheme == .light ? Color.Nym.surface : Color.Nym.background).ignoresSafeArea(edges: .top))
        .animation(.easeInOut(duration: 0.35), value: viewModel.shouldShowLogo)
    }
}

private extension AppFeatureView {
    @ViewBuilder
    func linkDestination(link: HomeLink, path: Binding<NavigationPath>) -> some View {
        switch link {
        case .settings:
            settingsDestination(path: path)
        case .entryGateways:
            GatewaysView(
                viewModel: GatewaysViewModel(
                    type: .entry,
                    path: path,
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
                    path: path,
                    appSettings: appSettings,
                    connectionManager: connectionManager,
                    gatewayManager: gatewayManager,
                    featureFlagsManager: featureFlagsManager
                )
            )
        case let .gatewayDetails(gateway: gateway, hopType: hopType):
            ServerDetailsView(
                path: path,
                gateway: gateway,
                hopType: hopType,
                externalLinkManager: externalLinkManager
            )
        default:
            EmptyView()
        }
    }

    @ViewBuilder
    func settingsDestination(path: Binding<NavigationPath>) -> some View {
#if os(iOS)
        SettingsView(viewModel: configuredIOSSettingsViewModel(path: path))
#elseif os(macOS)
        SettingsView(viewModel: configuredMacSettingsViewModel(path: path))
#endif
    }

#if os(iOS)
    func configuredIOSSettingsViewModel(path: Binding<NavigationPath>) -> SettingsViewModel {
        let settingsViewModel = SettingsViewModel(
            path: path,
            appSettings: appSettings,
            configurationManager: configurationManager,
            connectionManager: connectionManager,
            credentialsManager: credentialsManager,
            externalLinkManager: externalLinkManager,
            featureFlagsManager: featureFlagsManager,
            impactGenerator: impactGenerator,
            purchasesManager: purchasesManager
        )
        settingsViewModel.onSessionEvent = { [viewModel] event in
            viewModel.handleSessionEvent(event)
        }
        return settingsViewModel
    }
#elseif os(macOS)
    func configuredMacSettingsViewModel(path: Binding<NavigationPath>) -> SettingsViewModel {
        let settingsViewModel = SettingsViewModel(
            isServing: $grpcManager.isServing,
            path: path,
            appSettings: appSettings,
            configurationManager: configurationManager,
            connectionManager: connectionManager,
            credentialsManager: credentialsManager,
            externalLinkManager: externalLinkManager,
            featureFlagsManager: featureFlagsManager,
            impactGenerator: impactGenerator
        )
        settingsViewModel.onSessionEvent = { [viewModel] event in
            viewModel.handleSessionEvent(event)
        }
        return settingsViewModel
    }
#endif
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
