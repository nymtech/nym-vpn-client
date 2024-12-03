package net.nymtech.nymvpn.ui

import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.service.tunnel.TunnelManagerState
import net.nymtech.vpn.model.Country

data class AppUiState(
	val settings: Settings = Settings(),
	val gateways: Gateways = Gateways(),
	val managerState: TunnelManagerState = TunnelManagerState(),
	val entryCountry: Country = Country(isDefault = true),
	val exitCountry: Country = Country(isDefault = true),
)
