package net.nymtech.nymvpn.ui.screens.account.generating

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.toRoute
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.screens.auth.AuthRoute
import net.nymtech.nymvpn.ui.screens.auth.routeName
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.FreepassError
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.classifyFreepassError
import net.nymtech.nymvpn.util.ensureRegisteredAndApplyFreepass
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class GeneratingViewModel
@Inject
constructor(
	private val backendManager: BackendManager,
	savedStateHandle: SavedStateHandle,
	private val billingManager: BillingManager,
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-generate-account-vm"
	}

	val mode: GeneratingMode = GeneratingMode.valueOf(savedStateHandle.toRoute<Route.Generating>().mode)

	private val _error = MutableSharedFlow<Unit>(extraBufferCapacity = 1, onBufferOverflow = BufferOverflow.DROP_OLDEST)
	val error = _error.asSharedFlow()

	private val _pendingNavigation = MutableStateFlow<Route?>(null)
	val pendingNavigation = _pendingNavigation.asStateFlow()

	private val _freepassError = MutableStateFlow<FreepassError?>(null)
	val freepassError = _freepassError.asStateFlow()

	fun onFreepassErrorHandled() { _freepassError.value = null }

	private val code: String? = savedStateHandle.toRoute<Route.Generating>().code

	init {
		when (mode) {
			GeneratingMode.CreateAccount -> {
				viewModelScope.launch {
					val billingAvailable = checkBillingAvailable()
					Timber.tag(TAG).i("CreateAccountRequested billingAvailable=$billingAvailable")

					runCatching {
						backendManager.createAccount()
						Timber.tag(TAG).i("CreateAccountSuccess")

						if (billingAvailable) {
							_pendingNavigation.value = Route.SelectPlan
						} else {
							val shouldShowTechnical = !settingsRepository.isTechnicalOptScreenCompleted()
							_pendingNavigation.value = if (shouldShowTechnical) Route.Main(authRoute = AuthRoute.TechOpt.routeName) else Route.Main()
						}
					}.onFailure { t ->
						Timber.tag(TAG).e(t, "AccountSetupFailed")
						_error.emit(Unit)
						SnackbarController.showMessage(StringValue.StringResource(R.string.account_generating_error))
					}
				}
			}
			GeneratingMode.Freepass -> startFreepassFlow()
			GeneratingMode.DeepLinkLogin -> Timber.tag(TAG).i("Generating started in DeepLinkLogin mode")
		}
	}

	private fun startFreepassFlow() = viewModelScope.launch {
		val freepassCode = code
		if (freepassCode.isNullOrEmpty()) {
			Timber.tag(TAG).e("Freepass flow started without a code")
			_freepassError.value = FreepassError.GENERIC
			return@launch
		}
		runCatching {
			if (!backendManager.isMnemonicStored()) {
				backendManager.createAccount()
				Timber.tag(TAG).i("CreateAccountSuccess (freepass)")
			}
			// Onboarding path: the account was just created locally; register it with the API and
			// apply the code. (The Settings path uses the dedicated RedeemVoucher flow instead.)
			backendManager.ensureRegisteredAndApplyFreepass(freepassCode)
			Timber.tag(TAG).i("ApplyFreepassSuccess")
			val shouldShowTechnical = !settingsRepository.isTechnicalOptScreenCompleted()
			_pendingNavigation.value =
				if (shouldShowTechnical) Route.Main(authRoute = AuthRoute.TechOpt.routeName) else Route.Main()
		}.onFailure { t ->
			Timber.tag(TAG).w(t, "ApplyFreepassFailed")
			_freepassError.value = classifyFreepassError(t)
		}
	}

	private fun checkBillingAvailable(): Boolean {
		val billingAllowed = BuildConfig.APPLICATION_ID == Constants.APP_ID
		return billingAllowed && billingManager.isAvailable()
	}
}
