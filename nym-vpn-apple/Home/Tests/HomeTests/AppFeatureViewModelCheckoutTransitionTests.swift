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
final class AppFeatureViewModelCheckoutTransitionTests: XCTestCase {
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

    func testPlanPurchaseNavigationDefersWhileProcessingDrawerVisible() {
        let viewModel = makeViewModel()
        viewModel.handleSessionEvent(.authCompleted(outcome: .registeredActive, flow: .createAccount))
        XCTAssertEqual(viewModel.drawerContent, .processing)

        let tokenBefore = viewModel.planPurchaseNavigationToken
        viewModel.handleSessionEvent(.requestPlanPurchase)

        XCTAssertEqual(viewModel.planPurchaseNavigationToken, tokenBefore)
        XCTAssertEqual(viewModel.navigationIntent, .pushPlanPurchase)
    }

    func testDismissPostPurchaseProcessingWithoutActiveCheckoutClearsProcessingDrawer() {
        let viewModel = makeViewModel()
        viewModel.handleSessionEvent(.authCompleted(outcome: .registeredActive, flow: .createAccount))
        XCTAssertEqual(viewModel.drawerContent, .processing)

        viewModel.requestDismissPostPurchaseProcessing()

        XCTAssertNotEqual(viewModel.drawerContent, .processing)
    }

    func testDismissPostPurchaseProcessingClearsDeferredPlanPurchaseNavigation() {
        let viewModel = makeViewModel()
        viewModel.handleSessionEvent(.authCompleted(outcome: .registeredActive, flow: .createAccount))
        viewModel.handleSessionEvent(.requestPlanPurchase)
        XCTAssertEqual(viewModel.navigationIntent, .pushPlanPurchase)

        viewModel.requestDismissPostPurchaseProcessing()

        XCTAssertNil(viewModel.navigationIntent)
    }

    func testCheckoutDismissedClearsDeferredPlanPurchaseNavigation() {
        let viewModel = makeViewModel()
        viewModel.handleSessionEvent(.authCompleted(outcome: .registeredActive, flow: .createAccount))
        viewModel.handleSessionEvent(.requestPlanPurchase)
        XCTAssertEqual(viewModel.navigationIntent, .pushPlanPurchase)

        viewModel.handleSessionEvent(.checkoutDismissed)

        XCTAssertNil(viewModel.navigationIntent)
    }
}
