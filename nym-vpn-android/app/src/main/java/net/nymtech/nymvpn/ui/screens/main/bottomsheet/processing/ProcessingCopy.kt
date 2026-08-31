package net.nymtech.nymvpn.ui.screens.main.bottomsheet.processing

import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AuthRoute
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.routeName
import nym_vpn_lib_types.AccountControllerErrorStateReason
import nym_vpn_lib_types.AccountControllerState

enum class LoginProcessingUiPhase {
	Carousel,
	Welcome,
}

data class LoginProcessingCopy(val titleRes: Int, val subtitleRes: Int?)

object ProcessingCopy {
	const val LOGIN_PROGRESS_STEP_COUNT = 4
	const val LOGIN_INITIAL_PROGRESS_STEP = 2
	const val CAROUSEL_INITIAL_DWELL_MS = 2_500L
	const val CAROUSEL_TICK_MS = 2_500L
	const val CAROUSEL_STEP_ADVANCE_DELAY_MS = 2_000L
	const val CAROUSEL_TICK_COUNT = 3
	const val READY_WELCOME_MS = 2_000L
	const val LOGIN_PROCESSING_MIN_HEIGHT_DP = 320
	const val LOGIN_PROCESSING_TOP_PADDING_DP = 48
	const val LOGIN_PROCESSING_BOTTOM_PADDING_DP = 16
	const val LOGIN_PROCESSING_HORIZONTAL_PADDING_DP = 18
	const val LOGIN_PROCESSING_LOGO_STEP_SPACING_DP = 12
	const val STEP_BAR_INITIAL_DELAY_MS = 800L
	const val STEP_BAR_FILL_MS = 300
	const val STEP_BAR_INITIAL_PAUSE_MS = 1_000L
	const val STEP_BAR_FORWARD_PAUSE_MS = 1_000L
	const val CREDENTIALS_CAROUSEL_TICK_MS = 10_000L
	const val CREDENTIALS_CAROUSEL_STEP_COUNT = 2
	val CREDENTIALS_STEP_TWO_FORBIDDEN_TERMS = listOf("connect", "ready")

	fun carouselDurationMs(): Long = CAROUSEL_INITIAL_DWELL_MS +
		(CAROUSEL_TICK_COUNT - 1).toLong() * CAROUSEL_TICK_MS +
		CAROUSEL_TICK_MS

	fun textAdvancePrecedesStepBarTick(): Boolean = CAROUSEL_STEP_ADVANCE_DELAY_MS > STEP_BAR_FORWARD_PAUSE_MS

	fun loginProgressStepForCarouselIndex(carouselIndex: Int): Int = (LOGIN_INITIAL_PROGRESS_STEP + carouselIndex).coerceAtMost(LOGIN_PROGRESS_STEP_COUNT)

	fun setupCarouselPairRes(index: Int): LoginProcessingCopy {
		val subtitleRes = when (index.coerceIn(0, CAROUSEL_TICK_COUNT - 1)) {
			0 -> R.string.account_login_processing_setting_up_step2_subtitle
			1 -> R.string.account_login_processing_setting_up_step3_subtitle
			else -> R.string.account_login_processing_setting_up_step4_subtitle
		}
		return LoginProcessingCopy(R.string.account_login_processing_setting_up, subtitleRes)
	}

	fun processingCopyForPhase(
		phase: LoginProcessingUiPhase,
		accountState: AccountControllerState?,
		credentialsCarouselTick: Int = 0,
		setupCarouselIndex: Int = 0,
		setupCarouselFinished: Boolean = false,
	): LoginProcessingCopy = when (phase) {
		LoginProcessingUiPhase.Welcome ->
			LoginProcessingCopy(R.string.account_payment_welcome, subtitleRes = null)
		LoginProcessingUiPhase.Carousel ->
			when {
				!setupCarouselFinished -> setupCarouselPairRes(setupCarouselIndex)
				LoginReadiness.shouldShowCredentialsCopy(setupCarouselFinished, accountState) -> {
					val (titleRes, subtitleRes) = credentialsCarouselPairRes(credentialsCarouselTick)
					LoginProcessingCopy(titleRes, subtitleRes)
				}
				else -> setupCarouselPairRes(CAROUSEL_TICK_COUNT - 1)
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
		is AccountControllerState.Syncing -> R.string.account_login_processing_loading_credentials
		else -> R.string.account_login_processing_setting_up
	}

	fun failureMessageResForState(state: AccountControllerState): Int = when (state) {
		is AccountControllerState.Error ->
			when (state.v1) {
				is AccountControllerErrorStateReason.MaxDeviceReached -> R.string.max_devices_reached_title
				else -> R.string.account_generating_error
			}
		else -> R.string.account_generating_error
	}

	fun postLoginRoute(state: AccountControllerState, showTechnicalOpt: Boolean): Route = when {
		LoginReadiness.isInactiveSubscription(state) || state is AccountControllerState.PendingSubscription -> Route.SelectPlan
		showTechnicalOpt -> Route.Main(authRoute = AuthRoute.TechOpt.routeName)
		else -> Route.Main()
	}

	fun timeoutRoute(showTechnicalOpt: Boolean): Route = if (showTechnicalOpt) {
		Route.Main(authRoute = AuthRoute.TechOpt.routeName)
	} else {
		Route.Main()
	}
}
