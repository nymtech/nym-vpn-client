package net.nymtech.nymvpn.ui

import net.nymtech.connectivity.NetworkStatus
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import java.util.Locale

data class AppUiState(
	val settings: Settings = Settings(),
	val gateways: Gateways = Gateways(),
	val managerState: TunnelManagerState = TunnelManagerState(),
	val networkStatus: NetworkStatus = NetworkStatus.Unknown,
) {
	val entryPointCountry = when (val entry = settings.entryPoint) {
		is EntryPoint.Gateway -> gateways.entryGateways.firstOrNull { it.identity == entry.identity }?.twoLetterCountryISO ?: "unknown"
		is EntryPoint.Location -> entry.location
		else -> "unknown"
	}
	val exitPointCountry = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> "unknown"
		is ExitPoint.Gateway -> gateways.exitGateways.firstOrNull { it.identity == exit.identity }?.twoLetterCountryISO ?: "unknown"
		is ExitPoint.Location -> exit.location
	}

	val entryPointName: String = when (val entry = settings.entryPoint) {
		is EntryPoint.Gateway -> {
			gateways.entryGateways.firstOrNull { it.identity == entry.identity }?.name ?: entry.identity
		}
		is EntryPoint.Location -> Locale(entry.location, entry.location).displayCountry
		else -> with(Settings.DEFAULT_ENTRY_POINT.location) {
			Locale(this, this).displayCountry
		}
	}

	val exitPointName: String = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> exit.address
		is ExitPoint.Gateway -> {
			gateways.exitGateways.firstOrNull { it.identity == exit.identity }?.name ?: exit.identity
		}
		is ExitPoint.Location -> Locale(exit.location, exit.location).displayCountry
	}
}
