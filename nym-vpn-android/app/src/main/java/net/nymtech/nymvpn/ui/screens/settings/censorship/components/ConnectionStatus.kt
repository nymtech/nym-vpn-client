package net.nymtech.nymvpn.ui.screens.settings.censorship.components

import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.util.extensions.isQuicSupported
import net.nymtech.vpn.backend.Tunnel

enum class ConnectionStatus {
	Disconnected,
	FastQuicConnected,
	FastNonQuicConnected,
}

internal fun getConnectionStatus(hasQuicEnabled: Boolean, appUiState: AppUiState): ConnectionStatus {
	if (appUiState.settings.vpnMode != Tunnel.Mode.TWO_HOP_MIXNET) return ConnectionStatus.Disconnected
	return when (appUiState.managerState.tunnelState) {
		Tunnel.State.Up -> {
			val isQuicSupported = appUiState.managerState.connectionData?.entryGateway?.id?.let { connectedGatewayId ->
				appUiState.gateways.entryGateways.firstOrNull { it.identity == connectedGatewayId }?.isQuicSupported()
			} ?: false

			if (isQuicSupported && hasQuicEnabled) ConnectionStatus.FastQuicConnected else ConnectionStatus.FastNonQuicConnected
		}

		else -> ConnectionStatus.Disconnected
	}
}
