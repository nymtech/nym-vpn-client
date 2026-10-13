package net.nymtech.nymvpn.manager.billing

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import net.nymtech.nymvpn.manager.backend.BackendManager
import timber.log.Timber
import javax.inject.Inject
import kotlin.coroutines.cancellation.CancellationException
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

/** Not a singleton: each screen gets its own fresh, never-cached answer. Call [refresh] from the main thread. */
class TrialEligibilityRepository internal constructor(private val backendManager: BackendManager, private val timeout: Duration) {

	@Inject
	constructor(backendManager: BackendManager) : this(backendManager, TIMEOUT)

	companion object {
		private const val TAG = "trial-eligibility"
		private val TIMEOUT = 5.seconds
	}

	private val _state = MutableStateFlow<TrialEligibility>(TrialEligibility.Loading)
	val state: StateFlow<TrialEligibility> = _state.asStateFlow()

	private var job: Job? = null

	fun refresh(scope: CoroutineScope) {
		job?.cancel()
		_state.value = TrialEligibility.Loading
		job = scope.launch {
			val eligible = fetch()
			// A newer refresh replaced this one; don't overwrite its answer.
			ensureActive()
			_state.value = TrialEligibility.Resolved(eligible)
		}
	}

	// No stored account = fresh user, eligible without a backend call; otherwise only an explicit yes counts.
	private suspend fun fetch(): Boolean {
		val eligible = try {
			withTimeoutOrNull(timeout) {
				if (!backendManager.isMnemonicStored()) {
					Timber.tag(TAG).i("TrialEligibility eligible=true reason=no_stored_account")
					true
				} else {
					backendManager.isTrialEligible()
				}
			}
		} catch (e: CancellationException) {
			throw e
		} catch (e: Exception) {
			Timber.tag(TAG).w(e, "TrialEligibilityFailed")
			false
		}
		if (eligible == null) Timber.tag(TAG).w("TrialEligibilityTimedOut")
		Timber.tag(TAG).i("TrialEligibility eligible=%b", eligible == true)
		return eligible == true
	}
}
