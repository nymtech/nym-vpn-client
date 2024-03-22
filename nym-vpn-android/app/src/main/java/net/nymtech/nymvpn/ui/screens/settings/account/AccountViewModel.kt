package net.nymtech.nymvpn.ui.screens.settings.account

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class AccountViewModel @Inject constructor(
    private val settingsRepository: SettingsRepository
) : ViewModel() {

    val uiState = settingsRepository.settingsFlow.map {
        //TODO mocked for now
        AccountUiState(
            loading = false,
            devices = emptyList(),
            subscriptionDaysRemaining = 31,
            subscriptionTotalDays = 31
        )
    }.stateIn(viewModelScope,
        SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
        AccountUiState()
    )
}