package net.nymtech.nymvpn.ui

import net.nymtech.connectivity.NetworkStatus
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.nymvpn.util.extensions.toDisplayCountry
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint

data class AppUiState(
	val settings: Settings = Settings(),
	val gateways: Gateways = Gateways(),
	val managerState: TunnelManagerState = TunnelManagerState(),
	val networkStatus: NetworkStatus = NetworkStatus.Unknown,
) {

	val entryPointCountry = when (val entry = settings.entryPoint) {
		is EntryPoint.Gateway -> gateways.entryGateways.firstOrNull { it.identity == entry.identity }?.twoLetterCountryISO
		is EntryPoint.Country -> entry.twoLetterIsoCountryCode
		else -> null
	}
	val exitPointCountry = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> null
		is ExitPoint.Gateway -> gateways.exitGateways.firstOrNull { it.identity == exit.identity }?.twoLetterCountryISO
		is ExitPoint.Country -> exit.twoLetterIsoCountryCode
		else -> null
	}

	val entryPointGatewayName = when (val entry = settings.entryPoint) {
		is EntryPoint.Country -> {
			if (managerState.tunnelState == Tunnel.State.Up) {
				managerState.connectionData?.let { data ->
					gateways.entryGateways.firstOrNull { it.identity == data.entryGateway.id }?.name
				}
			} else {
				null
			}
		}
		else -> null
	}

	val exitPointGatewayName = when (val exit = settings.exitPoint) {
		is ExitPoint.Country -> {
			if (managerState.tunnelState == Tunnel.State.Up) {
				managerState.connectionData?.let { data ->
					gateways.exitGateways.firstOrNull { it.identity == data.exitGateway.id }?.name
				}
			} else {
				null
			}
		}
		else -> null
	}

	val entryPointName: String = when (val entry = settings.entryPoint) {
		is EntryPoint.Gateway -> {
			gateways.entryGateways.firstOrNull { it.identity == entry.identity }?.name ?: entry.identity
		}
		is EntryPoint.Country -> entry.toDisplayCountry()
		else -> Settings.DEFAULT_ENTRY_POINT.toDisplayCountry()
	}

	val exitPointName: String = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> exit.address
		is ExitPoint.Gateway -> {
			gateways.exitGateways.firstOrNull { it.identity == exit.identity }?.name ?: exit.identity
		}
		is ExitPoint.Country -> exit.toDisplayCountry()
		else -> Settings.DEFAULT_EXIT_POINT.toDisplayCountry()
	}

	val exitPointId = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> exit.address
		is ExitPoint.Gateway -> exit.identity
		is ExitPoint.Country -> exit.twoLetterIsoCountryCode.lowercase()
		else -> null
	}

	val entryPointId = when (val entry = settings.entryPoint) {
		is EntryPoint.Gateway -> entry.identity
		is EntryPoint.Country -> entry.twoLetterIsoCountryCode.lowercase()
		else -> null
	}
}
