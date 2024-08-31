package net.nymtech.nymvpn.ui

import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.vpn.backend.Tunnel

data class AppUiState(
    val settings: Settings = Settings(),
    val gateways: Gateways = Gateways(),
    val showLocationTooltip: Boolean = false,
    val state: Tunnel.State = Tunnel.State.Down,
)
