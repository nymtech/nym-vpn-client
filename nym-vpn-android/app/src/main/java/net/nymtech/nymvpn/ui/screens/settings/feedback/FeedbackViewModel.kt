package net.nymtech.nymvpn.ui.screens.settings.feedback

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class FeedbackViewModel @Inject constructor(
    private val dataStoreManager: DataStoreManager
) : ViewModel() {

    val isErrorReportingEnabled = dataStoreManager.preferencesFlow.map {
        (it?.get(DataStoreManager.ERROR_REPORTING) ?: false)
    }.stateIn(viewModelScope,
        SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
        false
    )

    //TODO enable Sentry
    fun onErrorReportingSelected(selected: Boolean) = viewModelScope.launch {
        dataStoreManager.saveToDataStore(DataStoreManager.ERROR_REPORTING, selected)
    }
}