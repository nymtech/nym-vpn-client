package net.nymtech.nymvpn.ui

import net.nymtech.connectivity.NetworkStatus
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.nymvpn.util.Constants.countryCodesForRegionSupport
import net.nymtech.nymvpn.util.extensions.toDisplayCountry
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
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
		is EntryPoint.Region -> gateways.entryGateways.firstOrNull { it.region.equals(entry.region, true) }?.twoLetterCountryISO
		else -> null
	}
	val exitPointCountry = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> null
		is ExitPoint.Gateway -> gateways.exitGateways.firstOrNull { it.identity == exit.identity }?.twoLetterCountryISO
		is ExitPoint.Country -> exit.twoLetterIsoCountryCode
		is ExitPoint.Region -> gateways.exitGateways.firstOrNull { it.region.equals(exit.region, true) }?.twoLetterCountryISO
		else -> null
	}

	val entryPointGateway = when (val entry = settings.entryPoint) {
		is EntryPoint.Country, is EntryPoint.Region -> {
			if (managerState.tunnelState == Tunnel.State.Up || managerState.tunnelState == Tunnel.State.EstablishingConnection) {
				managerState.connectionData?.let { data ->
					gateways.entryGateways.firstOrNull { it.identity == data.entryGateway.id }
				}
			} else {
				null
			}
		}
		is EntryPoint.Gateway -> gateways.entryGateways.firstOrNull { it.identity == entry.identity }
		else -> null
	}

	val exitPointGateway = when (val exit = settings.exitPoint) {
		is ExitPoint.Country, ExitPoint.Region -> {
			if (managerState.tunnelState == Tunnel.State.Up || managerState.tunnelState == Tunnel.State.EstablishingConnection) {
				managerState.connectionData?.let { data ->
					gateways.exitGateways.firstOrNull { it.identity == data.exitGateway.id }
				}
			} else {
				null
			}
		}
		is ExitPoint.Gateway -> gateways.exitGateways.firstOrNull { it.identity == exit.identity }
		else -> null
	}

	val entryPointName: String = when (val entry = settings.entryPoint) {
		is EntryPoint.Gateway -> {
			gateways.entryGateways.firstOrNull { it.identity == entry.identity }?.name ?: entry.identity
		}
		is EntryPoint.Country -> entry.toDisplayCountry()
		is EntryPoint.Region -> gateways.entryGateways.firstOrNull { it.region.equals(entry.region, true) }?.entryPointNameForRegion() ?: entry.region
		else -> Settings.DEFAULT_ENTRY_POINT.toDisplayCountry()
	}

	val exitPointName: String = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> exit.address
		is ExitPoint.Gateway -> {
			gateways.exitGateways.firstOrNull { it.identity == exit.identity }?.name ?: exit.identity
		}
		is ExitPoint.Country -> exit.toDisplayCountry()
		is ExitPoint.Region -> gateways.exitGateways.firstOrNull { it.region.equals(exit.region, true) }?.entryPointNameForRegion() ?: exit.region
		else -> Settings.DEFAULT_EXIT_POINT.toDisplayCountry()
	}

	val entryPointLocation: String? = when (val entry = settings.entryPoint) {
		is EntryPoint.Country -> entryPointGateway.serverLocationOnCountrySelection(entry.twoLetterIsoCountryCode)
		is EntryPoint.Gateway -> gateways.entryGateways.firstOrNull {
			it.identity == entry.identity
		}?.let { it.serverLocationOnGatewaySelection(it.twoLetterCountryISO.orEmpty()) }
		is EntryPoint.Region -> entryPointGateway?.serverLocationOnRegionSelection()
		else -> null
	}

	val exitPointLocation: String? = when (val exit = settings.exitPoint) {
		is ExitPoint.Country -> exitPointGateway.serverLocationOnCountrySelection(exit.twoLetterIsoCountryCode)
		is ExitPoint.Gateway -> gateways.exitGateways.firstOrNull {
			it.identity == exit.identity
		}?.let { it.serverLocationOnGatewaySelection(it.twoLetterCountryISO.orEmpty()) }
		is ExitPoint.Region -> exitPointGateway?.serverLocationOnRegionSelection()
		else -> null
	}

	val exitPointId = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> exit.address
		is ExitPoint.Gateway -> exit.identity
		is ExitPoint.Country -> exit.twoLetterIsoCountryCode.lowercase()
		is ExitPoint.Region -> exit.region
		else -> null
	}

	val entryPointId = when (val entry = settings.entryPoint) {
		is EntryPoint.Gateway -> entry.identity
		is EntryPoint.Country -> entry.twoLetterIsoCountryCode.lowercase()
		is EntryPoint.Region -> entry.region
		else -> null
	}
}

private fun NymGateway?.serverLocationOnCountrySelection(twoLetterIsoCountryCode: String): String? {
	val region = this?.region.takeIf { countryCodesForRegionSupport.contains(twoLetterIsoCountryCode.lowercase()) }
	return this?.let { listOfNotNull(it.city, region).joinToString(", ") + " (${it.name})" }
}

private fun NymGateway.serverLocationOnGatewaySelection(twoLetterIsoCountryCode: String): String? {
	val region = this.region.takeIf { countryCodesForRegionSupport.contains(twoLetterIsoCountryCode.lowercase()) }
	return listOfNotNull(this.city, region, toDisplayCountry(twoLetterIsoCountryCode)).joinToString(", ")
}

private fun NymGateway.serverLocationOnRegionSelection(): String? {
	return this.city.orEmpty() + " (${this.name})"
}

private fun NymGateway.entryPointNameForRegion(): String {
	return listOfNotNull(toDisplayCountry(this.twoLetterCountryISO.orEmpty()), region).joinToString(", ")
}
