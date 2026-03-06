package net.nymtech.nymvpn.ui.screens.settings.privacy

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import javax.inject.Inject

@HiltViewModel
class PrivacyViewModel @Inject constructor(private val vpnConfigRepository: VpnConfigRepository, private val settingsRepository: SettingsRepository) : ViewModel() {

	fun onNetworkStatsEnabled(enabled: Boolean) = viewModelScope.launch {
		settingsRepository.setStatisticsEnabled(enabled)
	}

	fun onMonitoringEnabled(enabled: Boolean) = viewModelScope.launch {
		vpnConfigRepository.apply(CoreVpnConfigUpdate.SetSentry(enabled))
	}
}
