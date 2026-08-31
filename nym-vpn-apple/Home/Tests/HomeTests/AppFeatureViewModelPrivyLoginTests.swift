import XCTest
@testable import Home
import AccountPrefetchGates
import AppSettings
import ConnectionManager
import CredentialsManager
import GatewayManager
import ImpactGenerator
import NetworkMonitor
import SnackbarManager

@MainActor
final class AppFeatureViewModelPrivyLoginTests: XCTestCase {
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

    func testInactiveLoginAuthCompletedStartsLoginProcessingDrawer() {
        let viewModel = makeViewModel()
        viewModel.drawerContent = .welcome

        viewModel.handleSessionEvent(
            .authCompleted(outcome: .registeredNeedsPurchase, flow: .login)
        )

        XCTAssertEqual(viewModel.drawerContent, .processing)
        XCTAssertEqual(viewModel.processingViewModel?.flow, .login)
    }

    func testAuthWillBeginWithPreImportedCredentialDoesNotRequireOnChange() {
        XCTAssertTrue(
            DrawerSessionPolicy.shouldBeginCredentialImportCompletionOnAuthWillBegin(
                completesOnCredentialImport: true,
                isCredentialImported: true,
                pendingAuthFlow: .login,
                authHandoffCompleted: false
            )
        )
    }
}
