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

#if os(iOS)
    func testRequestInactiveSubscriptionPurchasePresentsChoiceWithoutRoutingToIAP() {
        let viewModel = makeViewModel()
        let initialNavigationToken = viewModel.planPurchaseNavigationToken

        viewModel.requestInactiveSubscriptionPurchase()

        XCTAssertTrue(viewModel.isSubscriptionPurchaseChoiceDisplayed)
        XCTAssertEqual(viewModel.planPurchaseNavigationToken, initialNavigationToken)
        XCTAssertNil(viewModel.navigationIntent)
    }

    func testDismissSubscriptionPurchaseChoiceClosesDialogWithoutRouting() {
        let viewModel = makeViewModel()
        viewModel.requestInactiveSubscriptionPurchase()

        viewModel.dismissSubscriptionPurchaseChoice()

        XCTAssertFalse(viewModel.isSubscriptionPurchaseChoiceDisplayed)
        XCTAssertNil(viewModel.navigationIntent)
    }

    func testBeginInAppSubscriptionPurchaseRoutesToPlanPurchase() {
        let viewModel = makeViewModel()
        viewModel.requestInactiveSubscriptionPurchase()

        viewModel.beginInAppSubscriptionPurchase()

        XCTAssertFalse(viewModel.isSubscriptionPurchaseChoiceDisplayed)
        XCTAssertEqual(viewModel.navigationIntent, .pushPlanPurchase)
    }

    func testBeginWebSubscriptionPurchaseIncrementsTokenWithoutRoutingToIAP() {
        let viewModel = makeViewModel()
        let initialWebToken = viewModel.webSubscriptionPurchaseToken
        viewModel.requestInactiveSubscriptionPurchase()

        viewModel.beginWebSubscriptionPurchase()

        XCTAssertFalse(viewModel.isSubscriptionPurchaseChoiceDisplayed)
        XCTAssertEqual(viewModel.webSubscriptionPurchaseToken, initialWebToken + 1)
        XCTAssertNil(viewModel.navigationIntent)
    }
#else
    func testRequestInactiveSubscriptionPurchaseRoutesDirectlyToPlanPurchaseOnMacOS() {
        let viewModel = makeViewModel()

        viewModel.requestInactiveSubscriptionPurchase()

        XCTAssertFalse(viewModel.isSubscriptionPurchaseChoiceDisplayed)
        XCTAssertEqual(viewModel.navigationIntent, .pushPlanPurchase)
    }
#endif
}
