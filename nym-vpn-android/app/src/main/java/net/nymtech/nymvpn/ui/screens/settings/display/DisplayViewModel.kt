package net.nymtech.nymvpn.ui.screens.settings.display

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class DisplayViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
) : ViewModel() {
	val uiState =
		settingsRepository.settingsFlow.map {
			DisplayUiState(false, it.theme)
		}.stateIn(
			viewModelScope,
			SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
			DisplayUiState(),
		)

	fun onThemeChange(theme: Theme) = viewModelScope.launch {
		settingsRepository.setTheme(theme)
	}
}
