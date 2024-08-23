package net.nymtech.nymvpn.ui

import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.service.gateway.domain.Gateway
import net.nymtech.vpn.Tunnel
import java.time.Instant

data class AppUiState(
	val settings: Settings = Settings(),
	val gateways: Gateways = Gateways(),
	val showLocationTooltip: Boolean = false,
	val state: Tunnel.State = Tunnel.State.Down,
)
