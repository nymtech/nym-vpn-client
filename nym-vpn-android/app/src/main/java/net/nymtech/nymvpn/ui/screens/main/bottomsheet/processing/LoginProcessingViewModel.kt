package net.nymtech.nymvpn.ui.screens.main.bottomsheet.processing

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import kotlin.system.measureTimeMillis
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.ui.Route
import nym_vpn_lib_types.AccountControllerState
import timber.log.Timber
import javax.inject.Inject
import kotlin.time.Duration.Companion.milliseconds

@HiltViewModel
class LoginProcessingViewModel
@Inject
constructor(private val backendManager: BackendManager, private val settingsRepository: SettingsRepository) : ViewModel() {

	companion object {
		private const val TAG = "ui-login-processing-vm"
	}

	private val _uiPhase = MutableStateFlow(LoginProcessingUiPhase.Carousel)
	val uiPhase: StateFlow<LoginProcessingUiPhase> = _uiPhase.asStateFlow()

	private val _progressStep = MutableStateFlow(ProcessingCopy.LOGIN_INITIAL_PROGRESS_STEP)
	val progressStep: StateFlow<Int> = _progressStep.asStateFlow()

	private val _navigationRoute = MutableStateFlow<Route?>(null)
	val navigationRoute: StateFlow<Route?> = _navigationRoute.asStateFlow()

	private val _timedOut = MutableStateFlow(false)
	val timedOut: StateFlow<Boolean> = _timedOut.asStateFlow()

	private val _failureMessageRes = MutableStateFlow<Int?>(null)
	val failureMessageRes: StateFlow<Int?> = _failureMessageRes.asStateFlow()

	private val _accountState = MutableStateFlow<AccountControllerState?>(null)
	val accountState: StateFlow<AccountControllerState?> = _accountState.asStateFlow()

	private val _credentialsCarouselTick = MutableStateFlow(0)
	val credentialsCarouselTick: StateFlow<Int> = _credentialsCarouselTick.asStateFlow()

	private val _setupCarouselIndex = MutableStateFlow(0)
	val setupCarouselIndex: StateFlow<Int> = _setupCarouselIndex.asStateFlow()

	private val _setupCarouselFinished = MutableStateFlow(false)
	val setupCarouselFinished: StateFlow<Boolean> = _setupCarouselFinished.asStateFlow()

	private var processingJob: Job? = null

	fun startProcessing() {
		if (processingJob?.isActive == true) {
			Timber.tag(TAG).i("LoginProcessingStartSkipped reason=jobActive phase=%s", _uiPhase.value)
			return
		}
		if (_uiPhase.value == LoginProcessingUiPhase.Welcome) {
			Timber.tag(TAG).i("LoginProcessingStartSkipped reason=welcomePending phase=%s", _uiPhase.value)
			return
		}

		processingJob?.cancel()
		_uiPhase.value = LoginProcessingUiPhase.Carousel
		_progressStep.value = ProcessingCopy.LOGIN_INITIAL_PROGRESS_STEP
		_navigationRoute.value = null
		_timedOut.value = false
		_failureMessageRes.value = null
		_accountState.value = null
		_credentialsCarouselTick.value = 0
		_setupCarouselIndex.value = 0
		_setupCarouselFinished.value = false
		processingJob = viewModelScope.launch {
			Timber.tag(TAG).i("LoginProcessingStarted")

			coroutineScope {
				val accountStateObserver = launch {
					backendManager.stateFlow
						.map { it.accountState }
						.collect { state ->
							_accountState.value = state
						}
				}
				val credentialsCarousel = launch {
					runCredentialsCarouselTicks()
				}
				try {
					val carousel = async {
						val durationMs = measureTimeMillis { runCarousel() }
						Timber.tag(TAG).i("LoginProcessingCarouselCompleted durationMs=%d", durationMs)
					}
					val work = async {
						val (workResult, durationMs) = measureTimedWork()
						Timber.tag(TAG).i("LoginProcessingWorkCompleted durationMs=%d result=%s", durationMs, workResult::class.simpleName)
						workResult
					}
					val workResult = work.await()
					carousel.await()
					finishAfterCarouselAndWork(workResult, carouselFinished = true)
				} finally {
					accountStateObserver.cancel()
					credentialsCarousel.cancel()
				}
			}
		}
	}

	private suspend fun runCarousel() {
		_setupCarouselIndex.value = 0
		delay(ProcessingCopy.CAROUSEL_INITIAL_DWELL_MS)
		_progressStep.value = ProcessingCopy.loginProgressStepForCarouselIndex(0)
		repeat(ProcessingCopy.CAROUSEL_TICK_COUNT - 1) { index ->
			delay(ProcessingCopy.CAROUSEL_TICK_MS)
			_setupCarouselIndex.value = index + 1
			delay(ProcessingCopy.CAROUSEL_STEP_ADVANCE_DELAY_MS)
			_progressStep.value = ProcessingCopy.loginProgressStepForCarouselIndex(index + 1)
		}
		delay(ProcessingCopy.CAROUSEL_TICK_MS)
		_setupCarouselFinished.value = true
	}

	private suspend fun runCredentialsCarouselTicks() {
		var tick = 0
		while (true) {
			val state = _accountState.value
			if (LoginReadiness.shouldShowCredentialsCopy(_setupCarouselFinished.value, state)) {
				_credentialsCarouselTick.value = tick
				delay(ProcessingCopy.CREDENTIALS_CAROUSEL_TICK_MS)
				tick = (tick + 1).coerceAtMost(ProcessingCopy.CREDENTIALS_CAROUSEL_STEP_COUNT - 1)
			} else {
				tick = 0
				_credentialsCarouselTick.value = 0
				delay(250)
			}
		}
	}

	private suspend fun measureTimedWork(): Pair<LoginReadinessWorkResult, Long> {
		var result: LoginReadinessWorkResult = LoginReadinessWorkResult.TimedOut
		val durationMs = measureTimeMillis {
			result = runAccountReadinessWork()
		}
		return result to durationMs
	}

	private suspend fun runAccountReadinessWork(): LoginReadinessWorkResult {
		runCatching { backendManager.refreshAccount() }
			.onFailure { Timber.tag(TAG).w(it, "AccountRefreshFailed") }

		val immediate = backendManager.stateFlow.value.accountState
		LoginReadiness.resolveReadinessWorkResult(immediate)?.let { result ->
			when (result) {
				is LoginReadinessWorkResult.Success ->
					Timber.tag(TAG).i("LoginProcessingAlreadySettled state=%s", result.state)
				is LoginReadinessWorkResult.Failed ->
					Timber.tag(TAG).w("LoginProcessingFailedImmediate state=%s", result.state)
				LoginReadinessWorkResult.TimedOut -> Unit
			}
			return result
		}

		val terminalState = withTimeoutOrNull(LoginReadiness.READINESS_TIMEOUT_MS.milliseconds) {
			backendManager.stateFlow
				.map { it.accountState }
				.drop(1)
				.first { state ->
					LoginReadiness.loginPreparationWaitOutcome(state) != LoginPreparationWaitOutcome.ContinueWaiting
				}
		}

		if (terminalState == null) {
			Timber.tag(TAG).w("LoginProcessingTimedOut timeoutMs=%s", LoginReadiness.READINESS_TIMEOUT_MS)
			return LoginReadinessWorkResult.TimedOut
		}

		return LoginReadiness.resolveReadinessWorkResult(terminalState)
			?: LoginReadinessWorkResult.TimedOut
	}

	private suspend fun finishAfterCarouselAndWork(workResult: LoginReadinessWorkResult, carouselFinished: Boolean) {
		val showTechnicalOpt = !settingsRepository.isTechnicalOptScreenCompleted()

		if (!LoginReadiness.canAdvanceLoginNavigation(workSettled = true, carouselFinished = carouselFinished)) {
			return
		}

		when (workResult) {
			is LoginReadinessWorkResult.Success -> {
				Timber.tag(TAG).i("LoginProcessingSettled state=%s", workResult.state)
				_uiPhase.value = LoginProcessingUiPhase.Welcome
				_progressStep.value = ProcessingCopy.LOGIN_PROGRESS_STEP_COUNT
				Timber.tag(TAG).i("LoginProcessingWelcomePhaseStarted")
				delay(ProcessingCopy.READY_WELCOME_MS)
				val route = ProcessingCopy.postLoginRoute(workResult.state, showTechnicalOpt)
				Timber.tag(TAG).i("LoginProcessingWelcomePhaseCompleted route=%s", route)
				_navigationRoute.value = route
			}
			is LoginReadinessWorkResult.Failed -> {
				Timber.tag(TAG).w("LoginProcessingFailed state=%s", workResult.state)
				_failureMessageRes.value = ProcessingCopy.failureMessageResForState(workResult.state)
				_navigationRoute.value = ProcessingCopy.timeoutRoute(showTechnicalOpt)
			}
			LoginReadinessWorkResult.TimedOut -> {
				_timedOut.value = true
				_navigationRoute.value = ProcessingCopy.timeoutRoute(showTechnicalOpt)
			}
		}
	}

	fun consumeNavigationRoute() {
		_navigationRoute.value = null
	}

	fun consumeFailureMessageRes() {
		_failureMessageRes.value = null
	}

	internal suspend fun runReadinessWorkForTests(): LoginReadinessWorkResult = runAccountReadinessWork()

	internal suspend fun finishAfterReadinessWorkForTests(workResult: LoginReadinessWorkResult) {
		finishAfterCarouselAndWork(workResult, carouselFinished = true)
	}

	internal fun runCredentialsCarouselTickOnceForTests(accountState: AccountControllerState?, setupCarouselFinished: Boolean = true) {
		_accountState.value = accountState
		_setupCarouselFinished.value = setupCarouselFinished
		if (LoginReadiness.shouldShowCredentialsCopy(setupCarouselFinished, accountState)) {
			val nextTick = (_credentialsCarouselTick.value + 1)
				.coerceAtMost(ProcessingCopy.CREDENTIALS_CAROUSEL_STEP_COUNT - 1)
			_credentialsCarouselTick.value = nextTick
		} else {
			_credentialsCarouselTick.value = 0
		}
	}
}
