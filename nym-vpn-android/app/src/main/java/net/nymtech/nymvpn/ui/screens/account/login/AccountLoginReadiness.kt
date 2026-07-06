package net.nymtech.nymvpn.ui.screens.account.login

import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.screens.auth.AuthRoute
import net.nymtech.nymvpn.ui.screens.auth.routeName
import nym_vpn_lib_types.AccountControllerErrorStateReason
import nym_vpn_lib_types.AccountControllerState

enum class LoginProcessingUiPhase {
	Carousel,
	Welcome,
}

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

data class LoginProcessingCopy(val titleRes: Int, val subtitleRes: Int?)

object AccountLoginReadiness {
	const val READINESS_TIMEOUT_MS = 60_000L
	const val LOGIN_PROGRESS_STEP_COUNT = 4
	const val LOGIN_INITIAL_PROGRESS_STEP = 2
	const val CAROUSEL_TICK_MS = 2_000L
	const val CAROUSEL_TICK_COUNT = 3
	const val READY_WELCOME_MS = 2_000L
	const val LOGIN_PROCESSING_MIN_HEIGHT_DP = 320
	const val LOGIN_PROCESSING_TOP_PADDING_DP = 48
	const val LOGIN_PROCESSING_BOTTOM_PADDING_DP = 16
	const val LOGIN_PROCESSING_HORIZONTAL_PADDING_DP = 18
	const val LOGIN_PROCESSING_LOGO_STEP_SPACING_DP = 12
	const val STEP_BAR_INITIAL_DELAY_MS = 300L
	const val STEP_BAR_FILL_MS = 300
	const val STEP_BAR_INITIAL_PAUSE_MS = 300L
	const val STEP_BAR_FORWARD_PAUSE_MS = 1_000L
	const val CREDENTIALS_CAROUSEL_TICK_MS = 10_000L
	const val CREDENTIALS_CAROUSEL_STEP_COUNT = 2

	fun carouselDurationMs(): Long = CAROUSEL_TICK_MS * CAROUSEL_TICK_COUNT

	fun canAdvanceLoginNavigation(workSettled: Boolean, carouselFinished: Boolean): Boolean = workSettled && carouselFinished

	fun loginProgressStepForCarouselIndex(carouselIndex: Int): Int = (LOGIN_INITIAL_PROGRESS_STEP + carouselIndex).coerceAtMost(LOGIN_PROGRESS_STEP_COUNT)

	fun shouldShowWelcomePhase(settled: AccountControllerState): Boolean = isSettledForLogin(settled)

	fun processingCopyForPhase(phase: LoginProcessingUiPhase, accountState: AccountControllerState?, credentialsCarouselTick: Int = 0): LoginProcessingCopy = when (phase) {
		LoginProcessingUiPhase.Welcome ->
			LoginProcessingCopy(R.string.account_payment_welcome, subtitleRes = null)
		LoginProcessingUiPhase.Carousel ->
			if (accountState is AccountControllerState.RequestingZkNyms) {
				val (titleRes, subtitleRes) = credentialsCarouselPairRes(credentialsCarouselTick)
				LoginProcessingCopy(titleRes, subtitleRes)
			} else {
				LoginProcessingCopy(R.string.account_login_processing_setting_up, subtitleRes = null)
			}
	}

	fun credentialsCarouselPairRes(tickIndex: Int): Pair<Int, Int> = when (tickIndex.coerceAtLeast(0)) {
		0 ->
			R.string.account_login_processing_loading_credentials to
				R.string.account_login_processing_loading_credentials_subtitle
		else ->
			R.string.account_login_processing_almost_ready to
				R.string.account_login_processing_almost_ready_subtitle
	}

	fun processingTitleForPhase(phase: LoginProcessingUiPhase, accountState: AccountControllerState? = null): Int = processingCopyForPhase(phase, accountState).titleRes

	fun processingTitleForCarouselAccountState(state: AccountControllerState?): Int = when (state) {
		is AccountControllerState.RequestingZkNyms -> R.string.account_login_processing_loading_credentials
		else -> R.string.account_login_processing_setting_up
	}

	fun loginPreparationWaitOutcome(state: AccountControllerState): LoginPreparationWaitOutcome = when (state) {
		is AccountControllerState.ReadyToConnect,
		is AccountControllerState.Decentralised,
		is AccountControllerState.UpgradeMode,
		is AccountControllerState.PendingSubscription,
		-> LoginPreparationWaitOutcome.Prepared
		is AccountControllerState.Error -> when (state.v1) {
			is AccountControllerErrorStateReason.InactiveSubscription,
			is AccountControllerErrorStateReason.AccountStatusNotActive,
			-> LoginPreparationWaitOutcome.Prepared
			else -> LoginPreparationWaitOutcome.Failed
		}
		is AccountControllerState.Syncing,
		is AccountControllerState.RequestingZkNyms,
		-> LoginPreparationWaitOutcome.ContinueWaiting
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

	fun failureMessageResForState(state: AccountControllerState): Int = when (state) {
		is AccountControllerState.Error ->
			when (state.v1) {
				is AccountControllerErrorStateReason.MaxDeviceReached -> R.string.max_devices_reached_title
				else -> R.string.account_generating_error
			}
		else -> R.string.account_generating_error
	}

	fun isReadyToConnect(state: AccountControllerState): Boolean = state is AccountControllerState.ReadyToConnect ||
		state is AccountControllerState.Decentralised ||
		state is AccountControllerState.UpgradeMode

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

	fun postLoginRoute(state: AccountControllerState, showTechnicalOpt: Boolean): Route = when {
		isInactiveSubscription(state) || state is AccountControllerState.PendingSubscription -> Route.SelectPlan
		showTechnicalOpt -> Route.Main(authRoute = AuthRoute.TechOpt.routeName)
		else -> Route.Main()
	}

	fun timeoutRoute(showTechnicalOpt: Boolean): Route = if (showTechnicalOpt) {
		Route.Main(authRoute = AuthRoute.TechOpt.routeName)
	} else {
		Route.Main()
	}

	fun routeAfterDeeplinkCredentialStore(storeSucceeded: Boolean): Route = if (storeSucceeded) {
		Route.Main(autoStart = false, loginProcessing = true)
	} else {
		Route.Main(autoStart = false)
	}
}
