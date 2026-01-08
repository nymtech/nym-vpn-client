package net.nymtech.nymvpn.ui.screens.technical

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import javax.inject.Inject

@HiltViewModel
class TechnicalOptViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	fun onNetworkStatsEnabled(enabled: Boolean) = viewModelScope.launch {
		settingsRepository.setStatisticsEnabled(enabled)
	}

	fun onMonitoringEnabled(enabled: Boolean) = viewModelScope.launch {
		settingsRepository.setSentryMonitoring(enabled)
	}

	fun onContinueClicked() = viewModelScope.launch {
		settingsRepository.setTechnicalOptScreenCompleted()
	}
}
