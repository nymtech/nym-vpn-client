package net.nymtech.nymvpn.ui.screens.account.login

import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.screens.auth.AuthRoute
import net.nymtech.nymvpn.ui.screens.auth.routeName
import nym_vpn_lib_types.AccountControllerErrorStateReason
import nym_vpn_lib_types.AccountControllerState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class AccountLoginReadinessTest {

	@Test
	fun isSettledForLogin_onlyReadyOrInactive() {
		assertTrue(AccountLoginReadiness.isSettledForLogin(AccountControllerState.ReadyToConnect))
		assertTrue(AccountLoginReadiness.isSettledForLogin(AccountControllerState.PendingSubscription))
		assertTrue(
			AccountLoginReadiness.isSettledForLogin(
				AccountControllerState.Error(AccountControllerErrorStateReason.InactiveSubscription),
			),
		)
		assertFalse(AccountLoginReadiness.isSettledForLogin(AccountControllerState.Syncing))
		assertFalse(AccountLoginReadiness.isSettledForLogin(AccountControllerState.RequestingZkNyms))
		assertFalse(
			AccountLoginReadiness.isSettledForLogin(
				AccountControllerState.Error(
					AccountControllerErrorStateReason.ApiFailure(context = "login", details = "timeout"),
				),
			),
		)
		assertFalse(
			AccountLoginReadiness.isSettledForLogin(
				AccountControllerState.Error(AccountControllerErrorStateReason.MaxDeviceReached),
			),
		)
	}

	@Test
	fun readyStates_areReadyToConnect() {
		assertTrue(AccountLoginReadiness.isReadyToConnect(AccountControllerState.ReadyToConnect))
		assertTrue(AccountLoginReadiness.isReadyToConnect(AccountControllerState.Decentralised))
		assertTrue(AccountLoginReadiness.isReadyToConnect(AccountControllerState.UpgradeMode))
		assertFalse(AccountLoginReadiness.isReadyToConnect(AccountControllerState.Syncing))
	}

	@Test
	fun inactiveRoutesToSelectPlan() {
		val pending = AccountLoginReadiness.postLoginRoute(
			AccountControllerState.PendingSubscription,
			showTechnicalOpt = false,
		)
		assertEquals(Route.SelectPlan, pending)

		val inactive = AccountLoginReadiness.postLoginRoute(
			AccountControllerState.Error(
				AccountControllerErrorStateReason.InactiveSubscription,
			),
			showTechnicalOpt = true,
		)
		assertEquals(Route.SelectPlan, inactive)
	}

	@Test
	fun activeRoutesToMainOrTechOpt() {
		val main = AccountLoginReadiness.postLoginRoute(
			AccountControllerState.ReadyToConnect,
			showTechnicalOpt = false,
		)
		assertEquals(Route.Main(), main)

		val techOpt = AccountLoginReadiness.postLoginRoute(
			AccountControllerState.ReadyToConnect,
			showTechnicalOpt = true,
		)
		assertEquals(Route.Main(authRoute = AuthRoute.TechOpt.routeName), techOpt)
	}

	@Test
	fun timeoutProceedsToMain() {
		val main = AccountLoginReadiness.timeoutRoute(showTechnicalOpt = false)
		assertEquals(Route.Main(), main)

		val techOpt = AccountLoginReadiness.timeoutRoute(showTechnicalOpt = true)
		assertEquals(Route.Main(authRoute = AuthRoute.TechOpt.routeName), techOpt)
	}

	@Test
	fun canAdvanceLoginNavigation_requiresWorkAndCarousel() {
		assertFalse(AccountLoginReadiness.canAdvanceLoginNavigation(workSettled = true, carouselFinished = false))
		assertFalse(AccountLoginReadiness.canAdvanceLoginNavigation(workSettled = false, carouselFinished = true))
		assertTrue(AccountLoginReadiness.canAdvanceLoginNavigation(workSettled = true, carouselFinished = true))
	}

	@Test
	fun loginProgressStepForCarouselIndex_matchesIosSteps() {
		assertEquals(2, AccountLoginReadiness.loginProgressStepForCarouselIndex(0))
		assertEquals(3, AccountLoginReadiness.loginProgressStepForCarouselIndex(1))
		assertEquals(4, AccountLoginReadiness.loginProgressStepForCarouselIndex(2))
	}

	@Test
	fun shouldShowWelcomePhase_forEverySettledLoginState() {
		assertTrue(AccountLoginReadiness.shouldShowWelcomePhase(AccountControllerState.ReadyToConnect))
		assertTrue(AccountLoginReadiness.shouldShowWelcomePhase(AccountControllerState.PendingSubscription))
		assertTrue(
			AccountLoginReadiness.shouldShowWelcomePhase(
				AccountControllerState.Error(AccountControllerErrorStateReason.InactiveSubscription),
			),
		)
		assertFalse(AccountLoginReadiness.shouldShowWelcomePhase(AccountControllerState.Syncing))
	}

	@Test
	fun credentialsCarouselPairRes_tick0_and_tick1() {
		assertEquals(
			R.string.account_login_processing_loading_credentials to
				R.string.account_login_processing_loading_credentials_subtitle,
			AccountLoginReadiness.credentialsCarouselPairRes(0),
		)
		assertEquals(
			R.string.account_login_processing_almost_ready to
				R.string.account_login_processing_almost_ready_subtitle,
			AccountLoginReadiness.credentialsCarouselPairRes(1),
		)
		assertEquals(
			R.string.account_login_processing_almost_ready to
				R.string.account_login_processing_almost_ready_subtitle,
			AccountLoginReadiness.credentialsCarouselPairRes(99),
		)
	}

	@Test
	fun processingCopyForPhase_credentialsCarousel_includesSubtitles() {
		val tick0 = AccountLoginReadiness.processingCopyForPhase(
			LoginProcessingUiPhase.Carousel,
			AccountControllerState.RequestingZkNyms,
			credentialsCarouselTick = 0,
		)
		assertEquals(R.string.account_login_processing_loading_credentials, tick0.titleRes)
		assertEquals(R.string.account_login_processing_loading_credentials_subtitle, tick0.subtitleRes)

		val tick1 = AccountLoginReadiness.processingCopyForPhase(
			LoginProcessingUiPhase.Carousel,
			AccountControllerState.RequestingZkNyms,
			credentialsCarouselTick = 1,
		)
		assertEquals(R.string.account_login_processing_almost_ready, tick1.titleRes)
		assertEquals(R.string.account_login_processing_almost_ready_subtitle, tick1.subtitleRes)
	}

	@Test
	fun processingCopyForPhase_nonRequestingZkNyms_hasNoSubtitle() {
		val copy = AccountLoginReadiness.processingCopyForPhase(
			LoginProcessingUiPhase.Carousel,
			AccountControllerState.Syncing,
			credentialsCarouselTick = 1,
		)
		assertEquals(R.string.account_login_processing_setting_up, copy.titleRes)
		assertNull(copy.subtitleRes)
	}

	@Test
	fun processingTitleForPhase() {
		assertEquals(
			R.string.account_login_processing_setting_up,
			AccountLoginReadiness.processingTitleForPhase(LoginProcessingUiPhase.Carousel),
		)
		assertEquals(
			R.string.account_login_processing_loading_credentials,
			AccountLoginReadiness.processingTitleForPhase(
				LoginProcessingUiPhase.Carousel,
				AccountControllerState.RequestingZkNyms,
			),
		)
		assertEquals(
			R.string.account_payment_welcome,
			AccountLoginReadiness.processingTitleForPhase(LoginProcessingUiPhase.Welcome),
		)
	}

	@Test
	fun processingTitleForCarouselAccountState() {
		assertEquals(
			R.string.account_login_processing_setting_up,
			AccountLoginReadiness.processingTitleForCarouselAccountState(AccountControllerState.Syncing),
		)
		assertEquals(
			R.string.account_login_processing_loading_credentials,
			AccountLoginReadiness.processingTitleForCarouselAccountState(AccountControllerState.RequestingZkNyms),
		)
	}

	@Test
	fun loginPreparationWaitOutcome_matchesIosPolicy() {
		assertEquals(
			LoginPreparationWaitOutcome.Prepared,
			AccountLoginReadiness.loginPreparationWaitOutcome(AccountControllerState.ReadyToConnect),
		)
		assertEquals(
			LoginPreparationWaitOutcome.ContinueWaiting,
			AccountLoginReadiness.loginPreparationWaitOutcome(AccountControllerState.Syncing),
		)
		assertEquals(
			LoginPreparationWaitOutcome.ContinueWaiting,
			AccountLoginReadiness.loginPreparationWaitOutcome(AccountControllerState.RequestingZkNyms),
		)
		assertEquals(
			LoginPreparationWaitOutcome.Failed,
			AccountLoginReadiness.loginPreparationWaitOutcome(
				AccountControllerState.Error(AccountControllerErrorStateReason.MaxDeviceReached),
			),
		)
		assertEquals(
			LoginPreparationWaitOutcome.Prepared,
			AccountLoginReadiness.loginPreparationWaitOutcome(
				AccountControllerState.Error(AccountControllerErrorStateReason.InactiveSubscription),
			),
		)
	}

	@Test
	fun resolveReadinessWorkResult_maxDeviceReached_failsFast() {
		val failedState = AccountControllerState.Error(AccountControllerErrorStateReason.MaxDeviceReached)
		val result = AccountLoginReadiness.resolveReadinessWorkResult(failedState)
		assertTrue(result is LoginReadinessWorkResult.Failed)
		assertEquals(failedState, (result as LoginReadinessWorkResult.Failed).state)
	}

	@Test
	fun resolveReadinessWorkResult_syncing_returnsNull() {
		assertNull(AccountLoginReadiness.resolveReadinessWorkResult(AccountControllerState.Syncing))
	}

	@Test
	fun failureMessageResForState_maxDeviceReached() {
		val res = AccountLoginReadiness.failureMessageResForState(
			AccountControllerState.Error(AccountControllerErrorStateReason.MaxDeviceReached),
		)
		assertEquals(R.string.max_devices_reached_title, res)
	}

	@Test
	fun failureMessageResForState_apiFailure_usesGenericError() {
		val res = AccountLoginReadiness.failureMessageResForState(
			AccountControllerState.Error(
				AccountControllerErrorStateReason.ApiFailure(context = "login", details = "timeout"),
			),
		)
		assertEquals(R.string.account_generating_error, res)
	}

	@Test
	fun carouselDurationMs_matchesIosTicks() {
		assertEquals(6_000L, AccountLoginReadiness.carouselDurationMs())
	}

	@Test
	fun routeAfterDeeplinkCredentialStore_success_opensLoginProcessing() {
		val route = AccountLoginReadiness.routeAfterDeeplinkCredentialStore(storeSucceeded = true)
		assertEquals(Route.Main(autoStart = false, loginProcessing = true), route)
	}

	@Test
	fun routeAfterDeeplinkCredentialStore_failure_skipsLoginProcessing() {
		val route = AccountLoginReadiness.routeAfterDeeplinkCredentialStore(storeSucceeded = false)
		assertEquals(Route.Main(autoStart = false), route)
	}
}
