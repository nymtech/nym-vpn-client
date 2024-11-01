package net.nymtech.nymvpn.ui

import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Country

data class AppUiState(
	val settings: Settings = Settings(),
	val gateways: Gateways = Gateways(),
	val state: Tunnel.State = Tunnel.State.Down,
	val backendMessage: BackendMessage = BackendMessage.None,
	val isMnemonicStored: Boolean = false,
	val entryCountry: Country = Country(isLowLatency = true),
	val exitCountry: Country = Country(isDefault = true)
)
