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
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import net.nymtech.billing.model.BillingCode
import net.nymtech.billing.model.PurchaseState
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.ui.Route
import nym_vpn_lib_types.AccountControllerErrorStateReason
import nym_vpn_lib_types.AccountControllerState
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class PaymentViewModel
@Inject
constructor(
	private val billingManager: BillingManager,
	private val backendManager: BackendManager,
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-payment-vm"
		private const val ACCOUNT_STATE_POLL_INTERVAL_MS = 2_000L
	}

	private val _events = MutableSharedFlow<PaymentUiEvent>(
		replay = 0,
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	val events: SharedFlow<PaymentUiEvent> = _events.asSharedFlow()

	private val _accountState = MutableStateFlow<AccountControllerState?>(null)
	val accountState: StateFlow<AccountControllerState?> = _accountState.asStateFlow()

	private val _nextRoute = MutableStateFlow<Route?>(null)
	val nextRoute: StateFlow<Route?> = _nextRoute.asStateFlow()

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
						Timber.tag(TAG).d("PaymentPending")
						_events.tryEmit(PaymentUiEvent.PaymentPending)
					}

					val purchased = state.billingPurchase.firstOrNull { it.state == PurchaseState.PURCHASED }
					purchased?.let { purchase ->
						val token = purchase.token
						if (processedTokens.add(token)) {
							Timber.tag(TAG).i("PurchaseDetected state=PURCHASED action=register")

							viewModelScope.launch {
								runCatching {
									backendManager.registerAccount(token)
									backendManager.refreshAccount()

									_nextRoute.value = decidePostPaymentRoute()

									_events.tryEmit(PaymentUiEvent.PaymentSuccess)
									Timber.tag(TAG).i("PaymentRegisterSuccess")

									startAccountStatesUpdate()
								}.onFailure { e ->
									Timber.tag(TAG).e(e, "PaymentRegisterFailed")
									_events.tryEmit(PaymentUiEvent.PaymentError(e.message ?: "Register account failed"))
								}
							}
						} else {
							Timber.tag(TAG).d("PurchaseIgnored reason=already_processed")
						}
					}
				}

				state.billingInfo?.let { br ->
					when (br.responseCode) {
						BillingCode.OK -> {
							Timber.tag(TAG).d("BillingOk")
						}

						BillingCode.ITEM_ALREADY_OWNED -> {
							Timber.tag(TAG).i("SubscriptionAlreadyOwned")

							_nextRoute.value = decidePostPaymentRoute()

							_events.tryEmit(PaymentUiEvent.SubscriptionOwned)
							startAccountStatesUpdate()
						}

						BillingCode.USER_CANCELED -> {
							Timber.tag(TAG).i("BillingCanceled")
							_events.tryEmit(PaymentUiEvent.UserCanceled)
						}

						BillingCode.SERVICE_DISCONNECTED -> {
							Timber.tag(TAG).w("BillingServiceDisconnected")
						}

						BillingCode.SERVICE_UNAVAILABLE,
						BillingCode.BILLING_UNAVAILABLE,
						BillingCode.ERROR,
						BillingCode.NETWORK_ERROR,
						BillingCode.DEVELOPER_ERROR,
						BillingCode.FEATURE_NOT_SUPPORTED,
							-> {
							Timber.tag(TAG).w("BillingNonOk code=%s", br.responseCode)
							_events.tryEmit(PaymentUiEvent.PaymentError(br.debugMessage))
						}

						else -> {
							Timber.tag(TAG).d("BillingUnhandledCode code=%s", br.responseCode)
						}
					}
				}
			}
		}
	}

	private suspend fun decidePostPaymentRoute(): Route {
		val shouldShowTechnical = !settingsRepository.isTechnicalOptScreenCompleted()
		return if (shouldShowTechnical) Route.Technical else Route.Main()
	}

	fun startPurchaseFlow(activity: Activity, productId: String, userId: String?) {
		accountId = userId

		viewModelScope.launch {
			if (!accountId.isNullOrBlank()) {
				Timber.tag(TAG).i("PurchaseFlowRequested productId=%s", productId)
				billingManager.launchPurchaseFlow(activity, productId, accountId!!)
			} else {
				Timber.tag(TAG).w("PurchaseFlowRejected reason=missing_user_id productId=%s", productId)
				_events.tryEmit(PaymentUiEvent.PaymentError("Missing user id"))
			}
		}
	}

	fun refreshAccountState() {
		viewModelScope.launch {
			runCatching { backendManager.refreshAccountState() }
				.onFailure { Timber.tag(TAG).e(it, "AccountStateRefreshFailed") }
		}
	}

	private fun startAccountStatesUpdate() {
		stateUpdatesJob?.cancel()

		Timber.tag(TAG).d("AccountStatePollStart")

		stateUpdatesJob = viewModelScope.launch {
			while (isActive) {
				try {
					val state = backendManager.getAccountState()

					if (state is AccountControllerState.Error &&
						state.v1 == AccountControllerErrorStateReason.InactiveSubscription
					) {
						Timber.tag(TAG).i("AccountStateInactiveSubscription action=refresh")
						refreshAccountState()
					}

					_accountState.value = state

					if (state == AccountControllerState.ReadyToConnect) {
						Timber.tag(TAG).i("AccountStateReadyToConnect")
						break
					}
				} catch (t: Throwable) {
					Timber.tag(TAG).w(t, "AccountStatePollFailed")
				}

				delay(ACCOUNT_STATE_POLL_INTERVAL_MS)
			}

			Timber.tag(TAG).d("AccountStatePollStop")
		}
	}

	fun consumeNextRoute() {
		_nextRoute.value = null
	}
}
