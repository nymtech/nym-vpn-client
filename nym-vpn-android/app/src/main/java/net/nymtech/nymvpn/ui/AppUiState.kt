package net.nymtech.nymvpn.ui

import net.nymtech.connectivity.NetworkStatus
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.nymvpn.util.extensions.toDisplayCountry
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint

data class AppUiState(
	val settings: Settings = Settings(),
	val gateways: Gateways = Gateways(),
	val managerState: TunnelManagerState = TunnelManagerState(),
	val networkStatus: NetworkStatus = NetworkStatus.Unknown,
) {

	val entryPointCountry = when (val entry = settings.entryPoint) {
		is EntryPoint.Gateway -> gateways.entryGateways.firstOrNull { it.identity == entry.identity }?.twoLetterCountryISO
		is EntryPoint.Location -> entry.location
		else -> null
	}
	val exitPointCountry = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> null
		is ExitPoint.Gateway -> gateways.exitGateways.firstOrNull { it.identity == exit.identity }?.twoLetterCountryISO
		is ExitPoint.Location -> exit.location
	}

	val entryPointName: String = when (val entry = settings.entryPoint) {
		is EntryPoint.Gateway -> {
			gateways.entryGateways.firstOrNull { it.identity == entry.identity }?.name ?: entry.identity
		}
		is EntryPoint.Location -> entry.toDisplayCountry()
		else -> Settings.DEFAULT_ENTRY_POINT.toDisplayCountry()
	}

	val exitPointName: String = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> exit.address
		is ExitPoint.Gateway -> {
			gateways.exitGateways.firstOrNull { it.identity == exit.identity }?.name ?: exit.identity
		}
		is ExitPoint.Location -> exit.toDisplayCountry()
	}

	val exitPointId: String = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> exit.address
		is ExitPoint.Gateway -> exit.identity
		is ExitPoint.Location -> exit.location.lowercase()
	}

	val entryPointId = when (val entry = settings.entryPoint) {
		is EntryPoint.Gateway -> entry.identity
		is EntryPoint.Location -> entry.location.lowercase()
		else -> null
	}
}
