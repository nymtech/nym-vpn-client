package net.nymtech.nymvpn.ui.screens.settings.appearance.display

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.ui.theme.Theme
import javax.inject.Inject

@HiltViewModel
class DisplayViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	fun onThemeChange(theme: Theme) = viewModelScope.launch {
		settingsRepository.setTheme(theme)
	}
}
