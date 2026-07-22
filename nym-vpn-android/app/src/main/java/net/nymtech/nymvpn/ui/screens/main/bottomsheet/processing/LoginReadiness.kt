package net.nymtech.nymvpn.ui.screens.main.bottomsheet.processing

import nym_vpn_lib_types.AccountControllerErrorStateReason
import nym_vpn_lib_types.AccountControllerState

enum class LoginPreparationWaitOutcome {
	Prepared,
	ContinueWaiting,
	Failed,
}

sealed interface LoginReadinessWorkResult {
	data class Success(val state: AccountControllerState) : LoginReadinessWorkResult
	data class Failed(val state: AccountControllerState) : LoginReadinessWorkResult
	data object TimedOut : LoginReadinessWorkResult
}

object LoginReadiness {
	const val READINESS_TIMEOUT_MS = 60_000L

	fun loginPreparationWaitOutcome(state: AccountControllerState): LoginPreparationWaitOutcome = when (state) {
		is AccountControllerState.ReadyToConnect,
		is AccountControllerState.Decentralised,
		is AccountControllerState.PendingSubscription,
		-> LoginPreparationWaitOutcome.Prepared
		is AccountControllerState.Error -> when (state.v1) {
			is AccountControllerErrorStateReason.InactiveSubscription,
			is AccountControllerErrorStateReason.AccountStatusNotActive,
			-> LoginPreparationWaitOutcome.Prepared
			else -> LoginPreparationWaitOutcome.Failed
		}
		is AccountControllerState.Syncing -> LoginPreparationWaitOutcome.ContinueWaiting
		is AccountControllerState.Offline,
		is AccountControllerState.LoggedOut,
		-> LoginPreparationWaitOutcome.Failed
	}

	fun resolveReadinessWorkResult(state: AccountControllerState): LoginReadinessWorkResult? = when (loginPreparationWaitOutcome(state)) {
		LoginPreparationWaitOutcome.Prepared ->
			if (isSettledForLogin(state)) LoginReadinessWorkResult.Success(state) else null
		LoginPreparationWaitOutcome.Failed -> LoginReadinessWorkResult.Failed(state)
		LoginPreparationWaitOutcome.ContinueWaiting -> null
	}

	fun isReadyToConnect(state: AccountControllerState): Boolean = state is AccountControllerState.ReadyToConnect ||
		state is AccountControllerState.Decentralised

	fun isInactiveSubscription(state: AccountControllerState): Boolean = when (state) {
		is AccountControllerState.PendingSubscription -> true
		is AccountControllerState.Error ->
			state.v1 is AccountControllerErrorStateReason.InactiveSubscription ||
				state.v1 is AccountControllerErrorStateReason.AccountStatusNotActive
		else -> false
	}

	fun isSettledForLogin(state: AccountControllerState): Boolean = isReadyToConnect(state) ||
		isInactiveSubscription(state) ||
		state is AccountControllerState.PendingSubscription

	fun shouldShowWelcomePhase(settled: AccountControllerState): Boolean = isSettledForLogin(settled)

	fun canAdvanceLoginNavigation(workSettled: Boolean, carouselFinished: Boolean): Boolean = workSettled && carouselFinished

	fun shouldShowCredentialsCopy(setupCarouselFinished: Boolean, accountState: AccountControllerState?): Boolean = setupCarouselFinished && accountState is AccountControllerState.Syncing
}
