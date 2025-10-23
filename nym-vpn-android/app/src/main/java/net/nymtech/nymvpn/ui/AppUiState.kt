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
		else -> null
	}
	val exitPointCountry = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> null
		is ExitPoint.Gateway -> gateways.exitGateways.firstOrNull { it.identity == exit.identity }?.twoLetterCountryISO
		is ExitPoint.Country -> exit.twoLetterIsoCountryCode
		else -> null
	}

	val entryPointGateway = when (val entry = settings.entryPoint) {
		is EntryPoint.Country -> {
			if (managerState.tunnelState == Tunnel.State.Up || managerState.tunnelState == Tunnel.State.EstablishingConnection) {
				managerState.connectionData?.let { data ->
					gateways.entryGateways.firstOrNull { it.identity == data.entryGateway.id }
				}
			} else {
				null
			}
		}
		else -> null
	}

	val exitPointGateway = when (val exit = settings.exitPoint) {
		is ExitPoint.Country -> {
			if (managerState.tunnelState == Tunnel.State.Up || managerState.tunnelState == Tunnel.State.EstablishingConnection) {
				managerState.connectionData?.let { data ->
					gateways.exitGateways.firstOrNull { it.identity == data.exitGateway.id }
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
		is EntryPoint.Region -> gateways.entryGateways.firstOrNull { it.region == entry.region }?.entryPointNameForRegion(entry.region) ?: entry.region
		else -> Settings.DEFAULT_ENTRY_POINT.toDisplayCountry()
	}

	val exitPointName: String = when (val exit = settings.exitPoint) {
		is ExitPoint.Address -> exit.address
		is ExitPoint.Gateway -> {
			gateways.exitGateways.firstOrNull { it.identity == exit.identity }?.name ?: exit.identity
		}
		is ExitPoint.Country -> exit.toDisplayCountry()
		is ExitPoint.Region -> gateways.exitGateways.firstOrNull { it.region == exit.region }?.entryPointNameForRegion(exit.region) ?: exit.region
		else -> Settings.DEFAULT_EXIT_POINT.toDisplayCountry()
	}

	val entryPointLocation: String? = when (val entry = settings.entryPoint) {
		is EntryPoint.Country -> entryPointGateway.serverLocationOnCountrySelection(entry.twoLetterIsoCountryCode)
		is EntryPoint.Gateway -> gateways.entryGateways.firstOrNull {
			it.identity == entry.identity
		}?.let { it.serverLocationOnGatewaySelection(it.twoLetterCountryISO.orEmpty()) }
		is EntryPoint.Region -> gateways.entryGateways.firstOrNull { it.region == entry.region }?.serverLocationOnRegionSelection()
		else -> null
	}

	val exitPointLocation: String? = when (val exit = settings.exitPoint) {
		is ExitPoint.Country -> exitPointGateway.serverLocationOnCountrySelection(exit.twoLetterIsoCountryCode)
		is ExitPoint.Gateway -> gateways.exitGateways.firstOrNull {
			it.identity == exit.identity
		}?.let { it.serverLocationOnGatewaySelection(it.twoLetterCountryISO.orEmpty()) }
		is ExitPoint.Region -> gateways.exitGateways.firstOrNull { it.region == exit.region }?.serverLocationOnRegionSelection()
		else -> null
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

private fun NymGateway.entryPointNameForRegion(region: String): String {
	return listOfNotNull(toDisplayCountry(this.twoLetterCountryISO.orEmpty()), region).joinToString(", ")
}
