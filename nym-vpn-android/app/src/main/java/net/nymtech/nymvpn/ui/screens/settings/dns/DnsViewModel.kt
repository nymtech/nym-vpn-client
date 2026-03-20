package net.nymtech.nymvpn.ui.screens.settings.dns

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.ui.common.events.UiEvent
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import javax.inject.Inject

@HiltViewModel
class DnsViewModel @Inject constructor(private val backendManager: BackendManager, private val vpnConfigRepository: VpnConfigRepository) : ViewModel() {

	private val _defaultDns = MutableStateFlow<List<String>>(emptyList())
	val defaultDns: StateFlow<List<String>> = _defaultDns

	private val _customDns = MutableStateFlow<List<String>>(emptyList())
	val customDns: StateFlow<List<String>> = _customDns

	private val _backendUi = MutableStateFlow(DnsUiState())
	val backendUi = _backendUi.asStateFlow()

	private val _events = MutableSharedFlow<UiEvent>(
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	val events = _events.asSharedFlow()

	init {
		viewModelScope.launch {
			_defaultDns.value = DEFAULT_DNS_SERVERS
			_customDns.value = vpnConfigRepository.getConfig().customDns
		}

		viewModelScope.launch {
			backendManager.stateFlow.collect { s ->
				_backendUi.value = DnsUiState(
					tunnelState = s.tunnelState,
					isRestarting = s.isRestarting,
				)
			}
		}
	}

	fun onCustomDnsEnable(enabled: Boolean) = viewModelScope.launch {
		notifyReconnectIfConnected()
		vpnConfigRepository.apply(CoreVpnConfigUpdate.SetCustomDnsEnabled(enabled))
	}

	fun saveDnsListReconnectIfNeeded(list: List<String>) = viewModelScope.launch {
		notifyReconnectIfConnected()

		vpnConfigRepository.apply(CoreVpnConfigUpdate.SetCustomDns(list))
		_customDns.value = list
	}

	private fun notifyReconnectIfConnected() {
		val state = backendManager.getState()
		val isConnected = state == Tunnel.State.Up || state == Tunnel.State.EstablishingConnection

		if (isConnected) {
			_events.tryEmit(UiEvent.ReconnectStarted)
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
