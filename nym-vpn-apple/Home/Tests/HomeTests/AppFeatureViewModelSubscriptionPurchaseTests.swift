import XCTest
@testable import Home
import AppSettings
import ConnectionManager
import CredentialsManager
import GatewayManager
import ImpactGenerator
import NetworkMonitor
import SnackbarManager

@MainActor
final class AppFeatureViewModelSubscriptionPurchaseTests: XCTestCase {
    private func makeViewModel() -> AppFeatureViewModel {
#if os(iOS)
        AppFeatureViewModel(
            appSettings: .shared,
            credentialsManager: .shared,
            connectionManager: .shared,
            gatewayManager: .shared,
            snackbarManager: .shared,
            impactGenerator: .shared,
            networkMonitor: .shared
        )
#elseif os(macOS)
        AppFeatureViewModel(
            appSettings: .shared,
            credentialsManager: .shared,
            connectionManager: .shared,
            gatewayManager: .shared,
            snackbarManager: .shared,
            impactGenerator: .shared,
            networkMonitor: .shared,
            grpcManager: .shared
        )
#endif
    }

    func testRequestWelcomeOpensWelcomeDrawer() {
        let viewModel = makeViewModel()
        viewModel.drawerContent = .oneClick

        viewModel.handle(.requestWelcome)

        XCTAssertEqual(viewModel.pendingDrawerContent, .welcome)
        XCTAssertEqual(viewModel.drawerContent, .oneClick)

        viewModel.drawerTransitionCompleted()

        XCTAssertEqual(viewModel.drawerContent, .welcome)
        XCTAssertNil(viewModel.pendingDrawerContent)
    }

#if os(iOS)
    func testRequestInactiveSubscriptionPurchaseRoutesDirectlyToIAP() {
        let viewModel = makeViewModel()
        let initialWebToken = viewModel.webSubscriptionPurchaseToken

        viewModel.requestInactiveSubscriptionPurchase()

        XCTAssertEqual(viewModel.navigationIntent, .pushPlanPurchase)
        XCTAssertEqual(viewModel.webSubscriptionPurchaseToken, initialWebToken)
    }
#elseif os(macOS)
    func testRequestInactiveSubscriptionPurchaseStartsWebPurchaseOnMacOS() {
        let viewModel = makeViewModel()
        let initialWebToken = viewModel.webSubscriptionPurchaseToken

        viewModel.requestInactiveSubscriptionPurchase()

        XCTAssertEqual(viewModel.webSubscriptionPurchaseToken, initialWebToken + 1)
        XCTAssertNil(viewModel.navigationIntent)
    }
#endif
}
