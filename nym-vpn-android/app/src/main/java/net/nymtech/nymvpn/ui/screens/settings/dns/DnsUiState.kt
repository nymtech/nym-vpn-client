package net.nymtech.nymvpn.ui.screens.settings.dns

import net.nymtech.vpn.backend.Tunnel

data class DnsUiState(val tunnelState: Tunnel.State = Tunnel.State.Down, val isRestarting: Boolean = false)
