package net.nymtech.nymvpn.ui.screens.settings.developer

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import javax.inject.Inject

@HiltViewModel
class DeveloperViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	fun onManualGatewayOverride(enabled: Boolean) = viewModelScope.launch {
		settingsRepository.setManualGatewayOverride(
			enabled,
		)
	}

	fun onEntryGateway(gatewayId: String) = viewModelScope.launch {
		settingsRepository.setEntryGatewayId(gatewayId)
	}

	fun onExitGateway(gatewayId: String) = viewModelScope.launch {
		settingsRepository.setExitGatewayId(gatewayId)
	}
}
