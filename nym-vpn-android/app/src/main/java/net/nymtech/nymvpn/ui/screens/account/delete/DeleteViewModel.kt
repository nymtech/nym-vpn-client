package net.nymtech.nymvpn.ui.screens.account.delete

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.manager.billing.isPlayBillingSupported
import net.nymtech.nymvpn.util.extensions.toSubscriptionUiState
import net.nymtech.vpn.backend.Tunnel
import timber.log.Timber
import javax.inject.Inject
import kotlin.coroutines.cancellation.CancellationException
import kotlin.time.Duration.Companion.milliseconds

@HiltViewModel
class DeleteViewModel @Inject constructor(private val backendManager: BackendManager, private val billingManager: BillingManager, @IoDispatcher private val ioDispatcher: CoroutineDispatcher) :
	ViewModel() {

	companion object {
		private const val TAG = "ui-delete-account-vm"
		private const val BILLING_READY_TIMEOUT_MS = 5_000L
		private const val DISCONNECT_AWAIT_TIMEOUT_MS = 10_000L
	}

	private val _uiState = MutableStateFlow(DeleteUiState())
	val uiState: StateFlow<DeleteUiState> = _uiState.asStateFlow()

	private var subscriptionCheckJob: Job? = null

	fun checkSubscription() {
		if (_uiState.value.isDeleting || _uiState.value.step == DeleteStep.DELETED) return
		subscriptionCheckJob?.cancel()
		subscriptionCheckJob = viewModelScope.launch {
			val isPlayBilling = billingManager.isPlayBillingSupported()
			val (isPlayRenewing, timeLeftUntil) = try {
				if (isPlayBilling) awaitBillingReady()
				val isPlayRenewing = isPlayBilling && billingManager.hasRenewingSubscription()
				val isPlayCancelled = isPlayBilling && !isPlayRenewing && billingManager.hasActiveSubscription()
				val subscription = backendManager.accountSummaryFlow.value
					?.takeIf { it.isSubscriptionActive() }
					?.toSubscriptionUiState()
				isPlayRenewing to subscription?.takeIf { !it.isRecurring || isPlayCancelled }?.validUntilDate
			} catch (e: CancellationException) {
				throw e
			} catch (e: Exception) {
				Timber.tag(TAG).w(e, "SubscriptionCheckFailed")
				false to null
			}

			Timber.tag(TAG).i("SubscriptionChecked playRenewing=%b timeLeft=%b", isPlayRenewing, timeLeftUntil != null)
			_uiState.update {
				if (it.isDeleting || it.step == DeleteStep.DELETED) return@update it
				it.copy(
					isLoading = false,
					step = if (isPlayRenewing) DeleteStep.PLAY_STOP else DeleteStep.EXPLAINER,
					showPlayRecords = isPlayBilling,
					timeLeftUntil = timeLeftUntil,
				)
			}
		}
	}

	fun showConfirm() {
		_uiState.update { it.copy(showConfirm = true) }
	}

	fun dismissConfirm() {
		_uiState.update { it.copy(showConfirm = false) }
	}

	fun dismissError() {
		_uiState.update { it.copy(showError = false) }
	}

	fun deleteAccount() {
		if (_uiState.value.isDeleting) return
		subscriptionCheckJob?.cancel()
		_uiState.update { it.copy(showConfirm = false, isDeleting = true) }
		viewModelScope.launch(ioDispatcher) {
			Timber.tag(TAG).i("DeleteAccountRequested")
			try {
				stopTunnel()
				backendManager.deleteAccount()
			} catch (e: CancellationException) {
				throw e
			} catch (e: Exception) {
				Timber.tag(TAG).e(e, "DeleteAccountFailed")
				_uiState.update { it.copy(isDeleting = false, showError = true) }
				return@launch
			}
			Timber.tag(TAG).i("DeleteAccountSuccess")
			try {
				backendManager.refreshAccount()
			} catch (e: CancellationException) {
				throw e
			} catch (e: Exception) {
				Timber.tag(TAG).w(e, "RefreshAccountFailed")
			}
			_uiState.update { it.copy(isDeleting = false, step = DeleteStep.DELETED) }
		}
	}

	private suspend fun awaitBillingReady() {
		billingManager.initialize()
		val ready = withTimeoutOrNull(BILLING_READY_TIMEOUT_MS.milliseconds) {
			billingManager.uiState.first { billingManager.isReady() }
		}
		if (ready == null) Timber.tag(TAG).w("BillingNotReady")
	}

	private suspend fun stopTunnel() {
		if (backendManager.getState() == Tunnel.State.Down) return
		Timber.tag(TAG).i("DeleteAccountStoppingTunnel")
		backendManager.stopTunnel()
		val disconnected = withTimeoutOrNull(DISCONNECT_AWAIT_TIMEOUT_MS.milliseconds) {
			backendManager.stateFlow.first { it.tunnelState == Tunnel.State.Down }
		}
		if (disconnected == null) Timber.tag(TAG).w("DeleteAccountDisconnectTimeout")
	}
}
