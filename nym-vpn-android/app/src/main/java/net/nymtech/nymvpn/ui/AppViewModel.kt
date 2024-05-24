package net.nymtech.nymvpn.ui

import android.app.AlarmManager
import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings.ACTION_REQUEST_SCHEDULE_EXACT_ALARM
import android.widget.Toast
import androidx.annotation.RequiresApi
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
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.logcathelper.LogcatHelper
import net.nymtech.logcathelper.model.LogLevel
import net.nymtech.logcathelper.model.LogMessage
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SecretsRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.Native
import net.nymtech.nymvpn.service.gateway.GatewayService
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.FileUtils
import net.nymtech.nymvpn.util.NymVpnExceptions
import net.nymtech.nymvpn.util.log.NymLibException
import net.nymtech.vpn.VpnClient
import net.nymtech.vpn.model.Country
import timber.log.Timber
import java.time.Instant
import javax.inject.Inject
import javax.inject.Provider

@HiltViewModel
class AppViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val secretsRepository: Provider<SecretsRepository>,
	private val gatewayRepository: GatewayRepository,
	@Native private val gatewayService: GatewayService,
	private val vpnClient: Provider<VpnClient>,
) : ViewModel() {

	private val _uiState = MutableStateFlow(AppUiState())

	val logs = mutableStateListOf<LogMessage>()
	private val logsBuffer = mutableListOf<LogMessage>()

	init {
		viewModelScope.launch(Dispatchers.IO) {
			secretsRepository.get().credentialFlow.collect { cred ->
				cred?.let {
					getCredentialExpiry(it).onSuccess { expiry ->
						setIsNonExpiredCredentialImported(true)
						setCredentialExpiry(expiry)
					}.onFailure {
						setIsNonExpiredCredentialImported(false)
					}
				}
			}
		}
	}

	val uiState =
		combine(
			_uiState,
			settingsRepository.settingsFlow,
			vpnClient.get().stateFlow,
			secretsRepository.get().credentialFlow,
		) { state, settings, vpnState, cred ->
			AppUiState(
				false,
				state.snackbarMessage,
				state.snackbarMessageConsumed,
				vpnState,
				settings,
				isNonExpiredCredentialImported = state.isNonExpiredCredentialImported,
				credentialExpiryTime = state.credentialExpiryTime,
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

	private fun setCredentialExpiry(instant: Instant) {
		_uiState.update {
			it.copy(
				credentialExpiryTime = instant,
			)
		}
	}

	private fun setIsNonExpiredCredentialImported(value: Boolean) {
		_uiState.update {
			it.copy(
				isNonExpiredCredentialImported = value,
			)
		}
	}

	fun clearLogs() {
		logs.clear()
		logsBuffer.clear()
		LogcatHelper.clear()
	}

	suspend fun onValidCredentialCheck(): Result<Instant> {
		return withContext(viewModelScope.coroutineContext + Dispatchers.IO) {
			val credential = secretsRepository.get().getCredential()
			if (credential != null) {
				getCredentialExpiry(credential)
			} else {
				Result.failure(NymVpnExceptions.MissingCredentialException())
			}
		}
	}

	private suspend fun getCredentialExpiry(credential: String): Result<Instant> {
		return vpnClient.get().validateCredential(credential).onFailure {
			return Result.failure(NymVpnExceptions.InvalidCredentialException())
		}
	}

	fun saveLogsToFile(context: Context) {
		val fileName = "${Constants.BASE_LOG_FILE_NAME}-${Instant.now().epochSecond}.txt"
		val content = logs.joinToString(separator = "\n")
		FileUtils.saveFileToDownloads(context, content, fileName)
		showSnackbarMessage(context.getString(R.string.logs_saved))
	}

	fun setAnalyticsShown() = viewModelScope.launch {
		settingsRepository.setAnalyticsShown(true)
	}

	fun onEntryLocationSelected(selected: Boolean) = viewModelScope.launch(Dispatchers.IO) {
		settingsRepository.setFirstHopSelection(selected)
		settingsRepository.setFirstHopCountry(Country(isDefault = true))
// 		launch {
// 			setFirstHopToLowLatencyFromApi()
// 		}
// 		launch {
// 			setFirstHopToLowLatencyFromCache()
// 		}
	}

	private suspend fun setFirstHopToLowLatencyFromApi() {
		Timber.d("Updating low latency entry gateway")
		gatewayService.getLowLatencyCountry().onSuccess {
			Timber.d("New low latency gateway: $it")
			settingsRepository.setFirstHopCountry(it.copy(isLowLatency = true))
		}.onFailure {
			Timber.w(it)
		}
	}

	fun onErrorReportingSelected() = viewModelScope.launch {
		settingsRepository.setErrorReporting(!uiState.value.settings.errorReportingEnabled)
	}

	fun onAnalyticsReportingSelected() = viewModelScope.launch {
		settingsRepository.setAnalytics(!uiState.value.settings.analyticsEnabled)
	}

	private suspend fun setFirstHopToLowLatencyFromCache() {
		runCatching {
			gatewayRepository.getLowLatencyEntryCountry()
		}.onFailure {
			Timber.e(it)
		}.onSuccess {
			settingsRepository.setFirstHopCountry(it ?: Country(isDefault = true))
		}
	}

	fun openWebPage(url: String, context: Context) {
		try {
			val webpage: Uri = Uri.parse(url)
			Intent(Intent.ACTION_VIEW, webpage).apply {
				addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
			}.also {
				context.startActivity(it)
			}
		} catch (e: ActivityNotFoundException) {
			Timber.e(e)
			showSnackbarMessage(context.getString(R.string.no_browser_detected))
		}
	}

	@RequiresApi(Build.VERSION_CODES.S)
	fun isAlarmPermissionGranted(context: Context): Boolean {
		val alarmManager: AlarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
		return alarmManager.canScheduleExactAlarms()
	}

	@RequiresApi(Build.VERSION_CODES.S)
	fun requestAlarmPermission(context: Context) {
		val alarmManager: AlarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager

		when {
			alarmManager.canScheduleExactAlarms() -> {
				// permission granted
				Timber.d("Permission already granted for alarms")
			}
			else -> {
				// open alarm permission screen
				Intent().apply {
					addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
					action = ACTION_REQUEST_SCHEDULE_EXACT_ALARM
				}.also {
					context.startActivity(it)
				}
			}
		}
	}

	fun launchEmail(context: Context) {
		try {
			val intent =
				Intent(Intent.ACTION_SENDTO).apply {
					type = Constants.EMAIL_MIME_TYPE
					putExtra(
						Intent.EXTRA_EMAIL,
						arrayOf(context.getString(R.string.support_email)),
					)
					putExtra(
						Intent.EXTRA_SUBJECT,
						context.getString(R.string.email_subject),
					)
					addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
				}
			context.startActivity(
				Intent.createChooser(
					intent,
					context.getString(R.string.email_chooser),
				).apply {
					addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
				},
			)
		} catch (e: ActivityNotFoundException) {
			Timber.w(e)
			showSnackbarMessage(context.getString(R.string.no_email_detected))
		}
	}

	fun showSnackbarMessage(message: String) {
		_uiState.update {
			it.copy(
				snackbarMessage = message,
				snackbarMessageConsumed = false,
			)
		}
	}

	fun snackbarMessageConsumed() {
		_uiState.update {
			it.copy(
				snackbarMessage = "",
				snackbarMessageConsumed = true,
			)
		}
	}

	fun showFeatureInProgressMessage(context: Context) {
		Toast.makeText(
			context,
			context.getString(R.string.feature_in_progress),
			Toast.LENGTH_LONG,
		).show()
	}
}
