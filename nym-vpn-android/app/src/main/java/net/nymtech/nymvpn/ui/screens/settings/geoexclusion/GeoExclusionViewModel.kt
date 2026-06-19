package net.nymtech.nymvpn.ui.screens.settings.geoexclusion

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class GeoExclusionViewModel @Inject constructor(private val vpnConfigRepository: VpnConfigRepository) : ViewModel() {

	companion object {
		private const val TAG = "geo-exclusion-vm"
	}

	private val _failedToStart = MutableStateFlow(false)
	val failedToStart = _failedToStart.asStateFlow()

	fun onGeoExclusionEnabled(enabled: Boolean) = viewModelScope.launch {
		runCatching {
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetGeoExclusionEnabled(enabled))
			_failedToStart.value = false
		}.onFailure {
			Timber.tag(TAG).e(it, "Failed to update geo exclusion enabled")
			if (enabled) _failedToStart.value = true
		}
	}

	fun onGeoExclusionPortChanged(port: Int) = viewModelScope.launch {
		runCatching {
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetGeoExclusionPort(port))
		}.onFailure { Timber.tag(TAG).e(it, "Failed to update geo exclusion port") }
	}
}
