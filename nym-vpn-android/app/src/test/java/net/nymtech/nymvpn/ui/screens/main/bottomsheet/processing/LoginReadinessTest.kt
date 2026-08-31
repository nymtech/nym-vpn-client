package net.nymtech.nymvpn.ui.screens.main.bottomsheet.processing

import nym_vpn_lib_types.AccountControllerErrorStateReason
import nym_vpn_lib_types.AccountControllerState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class LoginReadinessTest {

	@Test
	fun isSettledForLogin_onlyReadyOrInactive() {
		assertTrue(LoginReadiness.isSettledForLogin(AccountControllerState.ReadyToConnect))
		assertTrue(LoginReadiness.isSettledForLogin(AccountControllerState.PendingSubscription))
		assertTrue(
			LoginReadiness.isSettledForLogin(
				AccountControllerState.Error(AccountControllerErrorStateReason.InactiveSubscription),
			),
		)
		assertFalse(LoginReadiness.isSettledForLogin(AccountControllerState.Syncing))
		assertFalse(
			LoginReadiness.isSettledForLogin(
				AccountControllerState.Error(
					AccountControllerErrorStateReason.ApiFailure(context = "login", details = "timeout"),
				),
			),
		)
		assertFalse(
			LoginReadiness.isSettledForLogin(
				AccountControllerState.Error(AccountControllerErrorStateReason.MaxDeviceReached),
			),
		)
	}

	@Test
	fun readyStates_areReadyToConnect() {
		assertTrue(LoginReadiness.isReadyToConnect(AccountControllerState.ReadyToConnect))
		assertTrue(LoginReadiness.isReadyToConnect(AccountControllerState.Decentralised))
		assertFalse(LoginReadiness.isReadyToConnect(AccountControllerState.Syncing))
	}

	@Test
	fun canAdvanceLoginNavigation_requiresWorkAndCarousel() {
		assertFalse(LoginReadiness.canAdvanceLoginNavigation(workSettled = true, carouselFinished = false))
		assertFalse(LoginReadiness.canAdvanceLoginNavigation(workSettled = false, carouselFinished = true))
		assertTrue(LoginReadiness.canAdvanceLoginNavigation(workSettled = true, carouselFinished = true))
	}

	@Test
	fun shouldShowWelcomePhase_forEverySettledLoginState() {
		assertTrue(LoginReadiness.shouldShowWelcomePhase(AccountControllerState.ReadyToConnect))
		assertTrue(LoginReadiness.shouldShowWelcomePhase(AccountControllerState.PendingSubscription))
		assertTrue(
			LoginReadiness.shouldShowWelcomePhase(
				AccountControllerState.Error(AccountControllerErrorStateReason.InactiveSubscription),
			),
		)
		assertFalse(LoginReadiness.shouldShowWelcomePhase(AccountControllerState.Syncing))
	}

	@Test
	fun shouldShowCredentialsCopy_afterSetupDuringBackendWait() {
		assertFalse(
			LoginReadiness.shouldShowCredentialsCopy(
				setupCarouselFinished = false,
				accountState = AccountControllerState.Syncing,
			),
		)
		assertTrue(
			LoginReadiness.shouldShowCredentialsCopy(
				setupCarouselFinished = true,
				accountState = AccountControllerState.Syncing,
			),
		)
		assertTrue(
			LoginReadiness.shouldShowCredentialsCopy(
				setupCarouselFinished = true,
				accountState = AccountControllerState.RequestingZkNyms,
			),
		)
	}

	@Test
	fun loginPreparationWaitOutcome_matchesIosPolicy() {
		assertEquals(
			LoginPreparationWaitOutcome.Prepared,
			LoginReadiness.loginPreparationWaitOutcome(AccountControllerState.ReadyToConnect),
		)
		assertEquals(
			LoginPreparationWaitOutcome.ContinueWaiting,
			LoginReadiness.loginPreparationWaitOutcome(AccountControllerState.Syncing),
		)
		assertEquals(
			LoginPreparationWaitOutcome.Failed,
			LoginReadiness.loginPreparationWaitOutcome(
				AccountControllerState.Error(AccountControllerErrorStateReason.MaxDeviceReached),
			),
		)
		assertEquals(
			LoginPreparationWaitOutcome.Prepared,
			LoginReadiness.loginPreparationWaitOutcome(
				AccountControllerState.Error(AccountControllerErrorStateReason.InactiveSubscription),
			),
		)
	}

	@Test
	fun resolveReadinessWorkResult_maxDeviceReached_failsFast() {
		val failedState = AccountControllerState.Error(AccountControllerErrorStateReason.MaxDeviceReached)
		val result = LoginReadiness.resolveReadinessWorkResult(failedState)
		assertTrue(result is LoginReadinessWorkResult.Failed)
		assertEquals(failedState, (result as LoginReadinessWorkResult.Failed).state)
	}

	@Test
	fun resolveReadinessWorkResult_syncing_returnsNull() {
		assertNull(LoginReadiness.resolveReadinessWorkResult(AccountControllerState.Syncing))
	}
}
