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
import net.nymtech.logcat_helper.LogcatHelper
import net.nymtech.logcat_helper.model.LogLevel
import net.nymtech.logcat_helper.model.LogMessage
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.service.gateway.GatewayApiService
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.FileUtils
import net.nymtech.nymvpn.util.log.NymLibException
import net.nymtech.vpn.model.Hop
import timber.log.Timber
import java.time.Instant
import java.util.Locale
import javax.inject.Inject


@HiltViewModel
class AppViewModel @Inject constructor(
    private val dataStoreManager: DataStoreManager,
    private val gatewayApiService: GatewayApiService,
    private val application: Application
) : ViewModel() {

    private val _uiState = MutableStateFlow(AppUiState())

    val logs = mutableStateListOf<LogMessage>()
    private val logsBuffer = mutableListOf<LogMessage>()

    val uiState = combine(_uiState, dataStoreManager.preferencesFlow) {
        state, preferences ->
        val theme : String = (preferences?.get(DataStoreManager.THEME)?.uppercase() ?: Theme.AUTOMATIC.name)
        val loggedIn : Boolean = preferences?.get(DataStoreManager.LOGGED_IN) ?: false
        AppUiState(false,  Theme.valueOf(theme), loggedIn, state.snackbarMessage, state.snackbarMessageConsumed)
    }.stateIn(viewModelScope,
        SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
        AppUiState()
    )

    fun readLogCatOutput() = viewModelScope.launch(viewModelScope.coroutineContext + Dispatchers.IO) {
        launch {
            LogcatHelper.logs {
                logsBuffer.add(it)
                when(it.level){
                    LogLevel.ERROR -> {
                        if(it.tag.contains(Constants.NYM_VPN_LIB_TAG))
                            Sentry.captureException(NymLibException("${it.time} - ${it.tag} ${it.message}"))
                    }
                    else -> Unit
                }
            }
        }
        launch {
            do {
                logs.addAll(logsBuffer)
                logsBuffer.clear()
                if(logs.size > Constants.LOG_BUFFER_SIZE) {
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
        viewModelScope.launch(Dispatchers.IO) {
            try {
                val gateways = gatewayApiService.getDescribedGateways()
                val countries = gateways.map {
                    val countryIso = it.bond.gateway.location
                    Hop.Country(countryIso, Locale(countryIso.lowercase(), countryIso).displayCountry)
                }.toSet()
                dataStoreManager.saveToDataStore(DataStoreManager.NODE_COUNTRIES, countries.toString())
            } catch (e : Exception) {
                Timber.e(e.message)
            }
        }
    }

    fun openWebPage(url: String) {
        try {
            val webpage: Uri = Uri.parse(url)
            val intent = Intent(Intent.ACTION_VIEW, webpage).apply {
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
                    putExtra(Intent.EXTRA_EMAIL, arrayOf(application.getString(R.string.support_email)))
                    putExtra(Intent.EXTRA_SUBJECT, application.getString(R.string.email_subject))
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                }
            application.startActivity(
                Intent.createChooser(intent, application.getString(R.string.email_chooser)).apply {
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                })
        } catch (e: ActivityNotFoundException) {
            Timber.e(e)
            showSnackbarMessage(application.getString(R.string.no_email_detected))
        }
    }
    fun showSnackbarMessage(message : String) {
        _uiState.value = _uiState.value.copy(
            snackbarMessage = message,
            snackbarMessageConsumed = false
        )
    }

    //TODO this should be package private
    fun snackbarMessageConsumed() {
        _uiState.value = _uiState.value.copy(
            snackbarMessage = "",
            snackbarMessageConsumed = true
        )
    }

    fun showFeatureInProgressMessage() {
        Toast.makeText(application.applicationContext, application.getString(R.string.feature_in_progress), Toast.LENGTH_LONG).show()
    }
}