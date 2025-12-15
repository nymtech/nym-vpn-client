package net.nymtech.nymvpn.ui.screens.settings.dns

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.vpn.backend.Tunnel
import javax.inject.Inject

@HiltViewModel
class DnsViewModel @Inject constructor(
	private val backendManager: BackendManager,
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	private val _defaultDns = MutableStateFlow<List<String>>(emptyList())
	val defaultDns: StateFlow<List<String>> = _defaultDns
	private val _customDns = MutableStateFlow<List<String>>(emptyList())
	val customDns: StateFlow<List<String>> = _customDns

	init {
		viewModelScope.launch {
			_defaultDns.value = DEFAULT_DNS_SERVERS
			_customDns.value = settingsRepository.getDnsList()
		}
	}

	fun onCustomDnsEnable(enabled: Boolean) = viewModelScope.launch {
		settingsRepository.setCustomDnsEnabled(enabled)
	}

	fun saveDnsList(list: List<String>) {
		viewModelScope.launch {
			settingsRepository.saveDnsList(list)
		}
	}

	fun reconnect() {
		viewModelScope.launch {
			if (backendManager.getState() == Tunnel.State.Up) {
				backendManager.stopTunnel()
				backendManager.startTunnel()
			}
		}
	}

	companion object {

		val DEFAULT_DNS_SERVERS = listOf(
			// Quad9
			"9.9.9.9",
			"149.112.112.112",
			"2620:fe::fe",
			"2620:fe::fe:9",

			// Cloudflare
			"1.1.1.1",
			"1.0.0.1",
			"2606:4700:4700::1111",
			"2606:4700:4700::1001",
		)
	}
}
