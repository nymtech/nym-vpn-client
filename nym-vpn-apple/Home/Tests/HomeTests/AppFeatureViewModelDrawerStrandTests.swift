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
final class AppFeatureViewModelDrawerStrandTests: XCTestCase {
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

    /// Regression for the pre-auth drawer strand.
    ///
    /// `.welcome`, `.processing` and `.technicalOptIns` share one `slideID`
    /// (`.preauth`), so a transition among them never changes `drawerSlideID`,
    /// never fires `DrawerView`'s slide, and therefore never runs
    /// `drawerTransitionCompleted()` — the only code that commits
    /// `pendingDrawerContent` into `drawerContent` and frees a finished
    /// `processingViewModel`. Moving from `.processing` to a pre-auth state must
    /// therefore commit `drawerContent` directly and free the processing VM,
    /// not merely stage `pendingDrawerContent` (which strands the drawer on a
    /// stale `.processing`).
    ///
    /// Mutation this kills: revert `stagePreauthDrawer` to stage-only
    /// (`pendingDrawerContent = content; return`) and `drawerContent` stays
    /// `.processing` — the first assertion fails.
    func testStagePreauthFromProcessingCommitsDirectlyAndFreesProcessingVM() {
        let viewModel = makeViewModel()

        // Deterministically land on a real `.processing` drawer.
        // (Mirrors AppFeatureViewModelCheckoutTransitionTests
        // .testLoginAuthCompletedStartsLoginProcessingDrawer.)
        viewModel.handleSessionEvent(.authCompleted(outcome: .loginReady, flow: .login))
        XCTAssertEqual(viewModel.drawerContent, .processing, "precondition: drawer is .processing")
        XCTAssertNotNil(viewModel.processingViewModel, "precondition: processing view-model present")

        viewModel.stagePreauthDrawer(.technicalOptIns)

        XCTAssertEqual(
            viewModel.drawerContent,
            .technicalOptIns,
            "pre-auth transition must commit drawerContent directly, not strand on .processing"
        )
        XCTAssertNil(viewModel.pendingDrawerContent, "must not be left staged")
        XCTAssertNil(viewModel.processingViewModel, "finished processing view-model must be freed")
    }
}
