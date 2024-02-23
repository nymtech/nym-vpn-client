package net.nymtech.nymvpn.ui.screens.settings.account

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.ui.screens.settings.account.model.Device
import net.nymtech.nymvpn.ui.screens.settings.account.model.DeviceType
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class AccountViewModel @Inject constructor(
    private val dataStoreManager: DataStoreManager
) : ViewModel() {

    val uiState = dataStoreManager.preferencesFlow.map {
        //TODO mock for now
        AccountUiState(
            loading = false,
            devices = listOf(Device("Sparrow", DeviceType.MAC_OS), Device("Falcon 1", DeviceType.ANDROID)),
            subscriptionDaysRemaining = 21,
            subscriptionTotalDays = 31
        )
    }.stateIn(viewModelScope,
        SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
        AccountUiState()
    )
}