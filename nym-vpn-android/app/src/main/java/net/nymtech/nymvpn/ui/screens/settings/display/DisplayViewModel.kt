package net.nymtech.nymvpn.ui.screens.settings.display

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class DisplayViewModel @Inject constructor(
    private val dataStoreManager: DataStoreManager
) : ViewModel() {

    val uiState = dataStoreManager.preferencesFlow.map {
        val theme : String = (it?.get(DataStoreManager.THEME)?.uppercase() ?: Theme.AUTOMATIC.name)
        DisplayUiState(false,  Theme.valueOf(theme))
    }.stateIn(viewModelScope,
        SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
        DisplayUiState()
    )

    fun onThemeChange(theme : Theme) = viewModelScope.launch {
        dataStoreManager.saveToDataStore(DataStoreManager.THEME, theme.name)
    }
}
