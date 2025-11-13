package net.nymtech.nymvpn.ui.screens.account.payment

import android.app.Activity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import net.nymtech.billing.model.BillingCode
import net.nymtech.billing.model.PurchaseState
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.billing.BillingManager
import nym_vpn_lib_types.AccountControllerState
import timber.log.Timber
import javax.inject.Inject
import kotlinx.coroutines.isActive
import nym_vpn_lib_types.AccountControllerErrorStateReason

@HiltViewModel
class PaymentViewModel
@Inject
constructor(
	private val billingManager: BillingManager,
	private val backendManager: BackendManager,
) : ViewModel() {

	private val _events = MutableSharedFlow<PaymentUiEvent>(
		replay = 0,
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	val events: SharedFlow<PaymentUiEvent> = _events.asSharedFlow()

	private val _accountState = MutableStateFlow<AccountControllerState?>(null)
	val accountState: StateFlow<AccountControllerState?> = _accountState.asStateFlow()

	private var accountId: String? = null
	private val processedTokens = mutableSetOf<String>()

	private var stateUpdatesJob: Job? = null

	init {
		billingManager.initialize()
		viewModelScope.launch {
			billingManager.uiState.collectLatest { state ->
				if (state.billingPurchase.isNotEmpty()) {
					val pending = state.billingPurchase.any { it.state == PurchaseState.PENDING }
					if (pending) {
						_events.tryEmit(PaymentUiEvent.PaymentPending)
					}
					val purchased = state.billingPurchase.firstOrNull {
						it.state == PurchaseState.PURCHASED
					}
					purchased?.let { purchase ->
						val token = purchase.token
						if (processedTokens.add(token)) {
							viewModelScope.launch {
								runCatching {
									backendManager.registerAccount(token)
									backendManager.refreshAccount()
									_events.tryEmit(PaymentUiEvent.PaymentSuccess)
									startAccountStatesUpdate()
								}.onFailure { e ->
									_events.tryEmit(PaymentUiEvent.PaymentError(e.message ?: "Register account failed"))
								}
							}
						} else {
							Timber.d("Purchase token handled")
						}
					}
				}
				state.billingInfo?.let { br ->
					when (br.responseCode) {
						BillingCode.OK -> {
							Timber.d("Billing OK: code=${br.responseCode}, msg=${br.debugMessage}")
						}
						BillingCode.ITEM_ALREADY_OWNED -> {
							Timber.d("Item already owned: ${br.debugMessage}")
							_events.tryEmit(PaymentUiEvent.SubscriptionOwned)
						}
						BillingCode.USER_CANCELED -> {
							Timber.w("User canceled: ${br.debugMessage}")
							_events.tryEmit(PaymentUiEvent.UserCanceled)
						}
						BillingCode.SERVICE_DISCONNECTED -> {
							Timber.w("Billing service disconnected: ${br.debugMessage}")
						}
						BillingCode.SERVICE_UNAVAILABLE,
						BillingCode.BILLING_UNAVAILABLE,
						BillingCode.ERROR,
						BillingCode.NETWORK_ERROR,
						BillingCode.DEVELOPER_ERROR,
						BillingCode.FEATURE_NOT_SUPPORTED,
						-> {
							Timber.e("Billing error ${br.responseCode}: ${br.debugMessage}")
							_events.tryEmit(PaymentUiEvent.PaymentError(br.debugMessage))
						}
						else -> {
							Timber.w("Unhandled billing code ${br.responseCode}: ${br.debugMessage}")
						}
					}
				}
			}
		}
	}

	fun startPurchaseFlow(activity: Activity, productId: String, userId: String?) {
		accountId = userId
		viewModelScope.launch {
			if (!accountId.isNullOrBlank()) {
				billingManager.launchPurchaseFlow(activity, productId, accountId!!)
			} else {
				_events.tryEmit(PaymentUiEvent.PaymentError("Missing user id"))
			}
		}
	}

	fun refreshAccountState() {
		viewModelScope.launch {
			backendManager.refreshAccountState()
		}
	}

	private fun startAccountStatesUpdate() {
		stateUpdatesJob?.cancel()
		stateUpdatesJob = viewModelScope.launch {
			while (isActive) {
				try {
					val state = backendManager.getAccountState()
					if (state is AccountControllerState.Error) {
						if (state.v1 == AccountControllerErrorStateReason.InactiveSubscription) {
							refreshAccountState()
						}
					}
					Timber.d("startAccountStatesUpdate $state")
					_accountState.value = state
					if (state == AccountControllerState.ReadyToConnect) {
						break
					}
				} catch (t: Throwable) {
					Timber.w(t, "Failed to get account state")
				}
				delay(2_000)
			}
		}
	}
}
