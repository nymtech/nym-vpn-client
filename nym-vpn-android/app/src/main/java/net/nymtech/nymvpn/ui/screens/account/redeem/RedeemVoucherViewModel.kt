package net.nymtech.nymvpn.ui.screens.account.redeem

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.toRoute
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.util.FreepassError
import net.nymtech.nymvpn.util.classifyFreepassError
import net.nymtech.nymvpn.util.ensureRegisteredAndApplyFreepass
import timber.log.Timber
import javax.inject.Inject

/**
 * Applies a free-pass voucher to the already-logged-in account (Settings → Redeem a voucher).
 * Unlike onboarding, this never creates an account; it only registers the existing account if
 * needed, then applies the code.
 */
@HiltViewModel
class RedeemVoucherViewModel
@Inject
constructor(
	private val backendManager: BackendManager,
	savedStateHandle: SavedStateHandle,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-redeem-voucher-vm"
	}

	sealed interface State {
		data object Applying : State
		data object Success : State
		data class Error(val kind: FreepassError) : State
	}

	private val code: String = savedStateHandle.toRoute<Route.RedeemVoucher>().code

	private val _state = MutableStateFlow<State>(State.Applying)
	val state = _state.asStateFlow()

	init {
		apply()
	}

	private fun apply() = viewModelScope.launch {
		_state.value = State.Applying
		runCatching {
			// Account already exists (this flow is only reachable when logged in) — do NOT create it.
			backendManager.ensureRegisteredAndApplyFreepass(code)
		}.onSuccess {
			Timber.tag(TAG).i("RedeemVoucherSuccess")
			_state.value = State.Success
		}.onFailure { t ->
			Timber.tag(TAG).w(t, "RedeemVoucherFailed")
			_state.value = State.Error(classifyFreepassError(t))
		}
	}
}
