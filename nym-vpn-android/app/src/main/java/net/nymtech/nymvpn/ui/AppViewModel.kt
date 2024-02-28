package net.nymtech.nymvpn.ui

import android.app.Application
import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
import androidx.core.content.ContextCompat
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.model.Country
import net.nymtech.nymvpn.service.gateway.GatewayApiService
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.Constants
import timber.log.Timber
import java.util.Locale
import javax.inject.Inject

@HiltViewModel
class AppViewModel @Inject constructor(
    private val dataStoreManager: DataStoreManager,
    private val gatewayApiService: GatewayApiService,
    private val application: Application
) : ViewModel() {

    private val _uiState = MutableStateFlow(AppUiState())

    val uiState = combine(_uiState, dataStoreManager.preferencesFlow) {
        state, preferences ->
        val theme : String = (preferences?.get(DataStoreManager.THEME)?.uppercase() ?: Theme.AUTOMATIC.name)
        val loggedIn : Boolean = preferences?.get(DataStoreManager.LOGGED_IN) ?: false
        AppUiState(false,  Theme.valueOf(theme), loggedIn, state.snackbarMessage, state.snackbarMessageConsumed)
    }.stateIn(viewModelScope,
        SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
        AppUiState()
    )

    fun updateCountryListCache() {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                val gateways = gatewayApiService.getDescribedGateways()
                val countries = gateways.map {
                    val countryIso = it.bond.gateway.location
                    Country(countryIso, Locale(countryIso.lowercase(), countryIso).displayCountry)
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
            ContextCompat.startActivity(
                application,
                Intent.createChooser(intent, application.getString(R.string.email_chooser)),
                null,
            )
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
}