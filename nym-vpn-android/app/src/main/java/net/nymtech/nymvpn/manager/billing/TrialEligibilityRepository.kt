package net.nymtech.nymvpn.manager.billing

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.manager.backend.BackendManager
import timber.log.Timber
import javax.inject.Inject
import kotlin.coroutines.cancellation.CancellationException

/** Not a singleton: each screen gets its own fresh, never-cached answer. Call [refresh] from the main thread. */
class TrialEligibilityRepository @Inject constructor(private val backendManager: BackendManager) {

	companion object {
		private const val TAG = "trial-eligibility"
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
			if (!backendManager.isMnemonicStored()) {
				Timber.tag(TAG).i("TrialEligibility eligible=true reason=no_stored_account")
				true
			} else {
				backendManager.isTrialEligible()
			}
		} catch (e: CancellationException) {
			throw e
		} catch (e: Exception) {
			Timber.tag(TAG).w(e, "TrialEligibilityFailed")
			false
		}
		Timber.tag(TAG).i("TrialEligibility eligible=%b", eligible)
		return eligible
	}
}
