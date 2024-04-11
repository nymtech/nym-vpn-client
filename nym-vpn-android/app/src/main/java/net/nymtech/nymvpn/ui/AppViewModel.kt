package net.nymtech.nymvpn.ui

import android.app.Application
import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
import android.widget.Toast
import androidx.compose.runtime.mutableStateListOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import io.sentry.Sentry
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.logcathelper.LogcatHelper
import net.nymtech.logcathelper.model.LogLevel
import net.nymtech.logcathelper.model.LogMessage
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.FileUtils
import net.nymtech.nymvpn.util.log.NymLibException
import net.nymtech.vpn.NymVpnClient
import net.nymtech.vpn.model.Country
import nym_vpn_lib.FfiException
import timber.log.Timber
import java.time.Instant
import javax.inject.Inject

@HiltViewModel
class AppViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val gatewayRepository: GatewayRepository,
	private val application: Application,
) : ViewModel() {
	private val _uiState = MutableStateFlow(AppUiState())

	val logs = mutableStateListOf<LogMessage>()
	private val logsBuffer = mutableListOf<LogMessage>()

	val uiState =
		combine(_uiState, settingsRepository.settingsFlow) { state, settings ->
			AppUiState(
				false,
				settings.theme,
				settings.loggedIn,
				state.snackbarMessage,
				state.snackbarMessageConsumed,
			)
		}.stateIn(
			viewModelScope,
			SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
			AppUiState(),
		)

	fun readLogCatOutput() = viewModelScope.launch(viewModelScope.coroutineContext + Dispatchers.IO) {
		launch {
			LogcatHelper.logs {
				logsBuffer.add(it)
				when (it.level) {
					LogLevel.ERROR -> {
						if (it.tag.contains(Constants.NYM_VPN_LIB_TAG)) {
							Sentry.captureException(
								NymLibException("${it.time} - ${it.tag} ${it.message}"),
							)
						}
					}

					else -> Unit
				}
			}
		}
		launch {
			do {
				logs.addAll(logsBuffer)
				logsBuffer.clear()
				if (logs.size > Constants.LOG_BUFFER_SIZE) {
					logs.removeRange(0, (logs.size - Constants.LOG_BUFFER_SIZE).toInt())
				}
				delay(Constants.LOG_BUFFER_DELAY)
			} while (true)
		}
	}

	fun clearLogs() {
		logs.clear()
		logsBuffer.clear()
		LogcatHelper.clear()
	}

	fun saveLogsToFile() {
		val fileName = "${Constants.BASE_LOG_FILE_NAME}-${Instant.now().epochSecond}.txt"
		val content = logs.joinToString(separator = "\n")
		FileUtils.saveFileToDownloads(application.applicationContext, content, fileName)
		showSnackbarMessage(application.getString(R.string.logs_saved))
	}

	fun updateCountryListCache() {
		Timber.i("Updating gateways country list")
		viewModelScope.launch(Dispatchers.IO) {
			updateExitCountriesCache()
			updateEntryCountriesCache()
			if (!settingsRepository.isFirstHopSelectionEnabled() || gatewayRepository.getFirstHopCountry().isDefault) {
				updateFirstHopDefaultCountry()
			}
		}
	}

	fun onEntryLocationSelected(selected: Boolean) = viewModelScope.launch {
		settingsRepository.setFirstHopSelection(selected)
		gatewayRepository.setFirstHopCountry(Country(isDefault = true))
		updateFirstHopDefaultCountry()
	}

	private suspend fun updateFirstHopDefaultCountry() {
		val firstHop = gatewayRepository.getFirstHopCountry()
		if (firstHop.isDefault || firstHop.isLowLatency) {
			setFirstHopToLowLatency()
		}
	}

	private suspend fun updateEntryCountriesCache() {
		try {
			val entryCountries = NymVpnClient.gateways(false)
			gatewayRepository.setEntryCountries(entryCountries)
		} catch (e: FfiException) {
			Timber.e(e)
		}
	}

	private suspend fun updateExitCountriesCache() {
		try {
			val exitCountries = NymVpnClient.gateways(true)
			gatewayRepository.setExitCountries(exitCountries)
		} catch (e: FfiException) {
			Timber.e(e)
		}
	}

	private suspend fun setFirstHopToLowLatency() {
		runCatching {
			NymVpnClient.getLowLatencyEntryCountryCode()
		}.onFailure {
			Timber.e(it)
		}.onSuccess {
			gatewayRepository.setFirstHopCountry(it)
		}
	}

	fun openWebPage(url: String) {
		try {
			val webpage: Uri = Uri.parse(url)
			val intent =
				Intent(Intent.ACTION_VIEW, webpage).apply {
					addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
				}
			application.startActivity(intent)
		} catch (e: ActivityNotFoundException) {
			Timber.e(e)
			showSnackbarMessage(application.getString(R.string.no_browser_detected))
		}
	}

	fun launchEmail() {
		try {
			val intent =
				Intent(Intent.ACTION_SENDTO).apply {
					type = Constants.EMAIL_MIME_TYPE
					putExtra(
						Intent.EXTRA_EMAIL,
						arrayOf(application.getString(R.string.support_email)),
					)
					putExtra(
						Intent.EXTRA_SUBJECT,
						application.getString(R.string.email_subject),
					)
					addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
				}
			application.startActivity(
				Intent.createChooser(
					intent,
					application.getString(R.string.email_chooser),
				).apply {
					addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
				},
			)
		} catch (e: ActivityNotFoundException) {
			Timber.e(e)
			showSnackbarMessage(application.getString(R.string.no_email_detected))
		}
	}

	fun showSnackbarMessage(message: String) {
		_uiState.value =
			_uiState.value.copy(
				snackbarMessage = message,
				snackbarMessageConsumed = false,
			)
	}

	// TODO this should be package private
	fun snackbarMessageConsumed() {
		_uiState.value =
			_uiState.value.copy(
				snackbarMessage = "",
				snackbarMessageConsumed = true,
			)
	}

	fun showFeatureInProgressMessage() {
		Toast.makeText(
			application.applicationContext,
			application.getString(R.string.feature_in_progress),
			Toast.LENGTH_LONG,
		).show()
	}
}
