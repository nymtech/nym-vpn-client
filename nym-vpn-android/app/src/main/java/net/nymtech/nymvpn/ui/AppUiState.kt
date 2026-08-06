package net.nymtech.nymvpn.ui

import net.nymtech.connectivity.NetworkStatus
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.nymvpn.ui.screens.settings.components.SubscriptionUiState
import net.nymtech.nymvpn.util.Constants.countryCodesForRegionSupport
import net.nymtech.nymvpn.util.extensions.toDisplayCountry
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.config.CoreVpnConfig
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint

data class AppUiState(
	val settings: Settings = Settings(),
	val gateways: Gateways = Gateways(),
	val vpnConfig: CoreVpnConfig = CoreVpnConfig(),
	val managerState: TunnelManagerState = TunnelManagerState(),
	val networkStatus: NetworkStatus = NetworkStatus.Unknown,
	val subscription: SubscriptionUiState? = null,
	val hasSubscriptionHistory: Boolean = false,
) {

	private val effectiveEntryPoint: EntryPoint
		get() {
			if (vpnConfig.entryPoint is EntryPoint.Random || vpnConfig.entryPoint is EntryPoint.Auto) {
				if (managerState.tunnelState != Tunnel.State.Down) {
					managerState.connectionData?.entryGateway?.id?.let { id ->
						return EntryPoint.Gateway(identity = id)
					}
				}
			}
			return vpnConfig.entryPoint
		}

	private val effectiveExitPoint: ExitPoint
		get() {
			if (vpnConfig.exitPoint is ExitPoint.Random || vpnConfig.exitPoint is ExitPoint.Auto) {
				if (managerState.tunnelState != Tunnel.State.Down) {
					managerState.connectionData?.exitGateway?.id?.let { id ->
						return ExitPoint.Gateway(identity = id)
					}
				}
			}
			return vpnConfig.exitPoint
		}

	val entryPointCountry = when (val entry = effectiveEntryPoint) {
		is EntryPoint.Gateway -> entryGateways().firstOrNull { it.identity == entry.identity }?.twoLetterCountryISO
		is EntryPoint.Country -> entry.twoLetterIsoCountryCode
		is EntryPoint.Region -> entryGateways().firstOrNull { it.region.equals(entry.region, true) }?.twoLetterCountryISO
		else -> null
	}

	val exitPointCountry = when (val exit = effectiveExitPoint) {
		is ExitPoint.Address -> null
		is ExitPoint.Gateway -> exitGateways().firstOrNull { it.identity == exit.identity }?.twoLetterCountryISO
		is ExitPoint.Country -> exit.twoLetterIsoCountryCode
		is ExitPoint.Region -> exitGateways().firstOrNull { it.region.equals(exit.region, true) }?.twoLetterCountryISO
		else -> null
	}

	val entryPointGateway = when (val entry = effectiveEntryPoint) {
		is EntryPoint.Country, is EntryPoint.Region -> {
			if (managerState.tunnelState == Tunnel.State.Up || managerState.tunnelState == Tunnel.State.EstablishingConnection) {
				managerState.connectionData?.let { data ->
					entryGateways().firstOrNull { it.identity == data.entryGateway.id }
				}
			} else {
				null
			}
		}
		is EntryPoint.Gateway -> entryGateways().firstOrNull { it.identity == entry.identity }
		else -> null
	}

	val exitPointGateway = when (val exit = effectiveExitPoint) {
		is ExitPoint.Country, is ExitPoint.Region -> {
			if (managerState.tunnelState == Tunnel.State.Up || managerState.tunnelState == Tunnel.State.EstablishingConnection) {
				managerState.connectionData?.let { data ->
					exitGateways().firstOrNull { it.identity == data.exitGateway.id }
				}
			} else {
				null
			}
		}
		is ExitPoint.Gateway -> exitGateways().firstOrNull { it.identity == exit.identity }
		else -> null
	}

	val entryPointName: String = when (val entry = effectiveEntryPoint) {
		is EntryPoint.Gateway -> {
			entryGateways().firstOrNull { it.identity == entry.identity }?.name ?: entry.identity
		}
		is EntryPoint.Country -> entry.toDisplayCountry()
		is EntryPoint.Region -> entryGateways().firstOrNull { it.region.equals(entry.region, true) }?.entryPointNameForRegion() ?: entry.region
		is EntryPoint.Auto -> "Safest"
		else -> "Random"
	}

	val exitPointName: String = when (val exit = effectiveExitPoint) {
		is ExitPoint.Address -> exit.address
		is ExitPoint.Gateway -> {
			exitGateways().firstOrNull { it.identity == exit.identity }?.name ?: exit.identity
		}
		is ExitPoint.Country -> exit.toDisplayCountry()
		is ExitPoint.Region -> exitGateways().firstOrNull { it.region.equals(exit.region, true) }?.entryPointNameForRegion() ?: exit.region
		is ExitPoint.Auto -> "Safest"
		else -> "Random"
	}

	val entryPointLocation: String? = when (val entry = effectiveEntryPoint) {
		is EntryPoint.Country -> entryPointGateway.serverLocationOnCountrySelection(entry.twoLetterIsoCountryCode)
		is EntryPoint.Gateway -> entryGateways().firstOrNull {
			it.identity == entry.identity
		}?.let { it.serverLocationOnGatewaySelection(it.twoLetterCountryISO.orEmpty()) }
		is EntryPoint.Region -> entryPointGateway?.serverLocationOnRegionSelection()
		else -> null
	}

	val exitPointLocation: String? = when (val exit = effectiveExitPoint) {
		is ExitPoint.Country -> exitPointGateway.serverLocationOnCountrySelection(exit.twoLetterIsoCountryCode)
		is ExitPoint.Gateway -> exitGateways().firstOrNull {
			it.identity == exit.identity
		}?.let { it.serverLocationOnGatewaySelection(it.twoLetterCountryISO.orEmpty()) }
		is ExitPoint.Region -> exitPointGateway?.serverLocationOnRegionSelection()
		else -> null
	}

	val exitPointId = when (val exit = effectiveExitPoint) {
		is ExitPoint.Address -> exit.address
		is ExitPoint.Gateway -> exit.identity
		is ExitPoint.Country -> exit.twoLetterIsoCountryCode.lowercase()
		is ExitPoint.Region -> exit.region
		else -> null
	}

	val entryPointId = when (val entry = effectiveEntryPoint) {
		is EntryPoint.Gateway -> entry.identity
		is EntryPoint.Country -> entry.twoLetterIsoCountryCode.lowercase()
		is EntryPoint.Region -> entry.region
		else -> null
	}

	private fun entryGateways(): List<NymGateway> = when (vpnConfig.mode) {
		Tunnel.Mode.FIVE_HOP_MIXNET -> gateways.entryGateways
		Tunnel.Mode.TWO_HOP_MIXNET -> gateways.wgGateways
	}

	private fun exitGateways(): List<NymGateway> = when (vpnConfig.mode) {
		Tunnel.Mode.FIVE_HOP_MIXNET -> gateways.exitGateways
		Tunnel.Mode.TWO_HOP_MIXNET -> gateways.wgGateways
	}
}

private fun NymGateway?.serverLocationOnCountrySelection(twoLetterIsoCountryCode: String): String? {
	val region = this?.region.takeIf { countryCodesForRegionSupport.contains(twoLetterIsoCountryCode.lowercase()) }
	return this?.let { listOfNotNull(it.city, region).joinToString(", ") + " (${it.name})" }
}

private fun NymGateway.serverLocationOnGatewaySelection(twoLetterIsoCountryCode: String): String {
	val region = this.region.takeIf { countryCodesForRegionSupport.contains(twoLetterIsoCountryCode.lowercase()) }
	return listOfNotNull(this.city, region, toDisplayCountry(twoLetterIsoCountryCode)).joinToString(", ")
}

private fun NymGateway.serverLocationOnRegionSelection(): String = this.city.orEmpty() + " (${this.name})"

private fun NymGateway.entryPointNameForRegion(): String = listOfNotNull(toDisplayCountry(this.twoLetterCountryISO.orEmpty()), region).joinToString(", ")
