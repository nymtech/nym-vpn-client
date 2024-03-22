package net.nymtech.nymvpn.ui.screens.settings.feedback

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class FeedbackViewModel @Inject constructor(
    private val settingsRepository: SettingsRepository
) : ViewModel() {

    val isErrorReportingEnabled = settingsRepository.settingsFlow.map {
        it.errorReportingEnabled
    }.stateIn(viewModelScope,
        SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
        false
    )

    fun onErrorReportingSelected(selected: Boolean) = viewModelScope.launch {
        settingsRepository.setErrorReporting(selected)
        //TODO prompt user to restart app
    }
}