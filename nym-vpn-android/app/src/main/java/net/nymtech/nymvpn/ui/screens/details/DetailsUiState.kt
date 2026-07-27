package net.nymtech.nymvpn.ui.screens.details

import net.nymtech.nymvpn.util.extensions.isQuicSupported
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.AsnKind
import nym_vpn_lib_types.NodeIdentity
import nym_vpn_lib_types.Score
import java.util.Locale

data class DetailsUiState(
	val identity: NodeIdentity = "",
	val name: String = "",
	val location: String = "",
	val description: String? = null,
	val countryCode: String? = null,
	val mixnetScore: Score? = null,
	val score: Score? = null,
	val load: Score? = null,
	val uptime: Float? = null,
	val lastUpdated: String? = null,
	val asn: String? = null,
	val asnName: String? = null,
	val asnKind: AsnKind? = null,
	val buildVersion: String? = null,
	val exitIpv4: String? = null,
	val exitIpv6: String? = null,
	val isQuickSupportedByGateway: Boolean = false,
	val nodeFamilyName: String? = null,
	val isPostQuantumEnabled: Boolean = false,
	val isFavorite: Boolean = false,
) {
	companion object {
		fun from(gateway: NymGateway): DetailsUiState {
			val country = Locale(gateway.twoLetterCountryISO.orEmpty(), gateway.twoLetterCountryISO.orEmpty()).displayCountry
			return DetailsUiState(
				identity = gateway.identity,
				name = gateway.name,
				description = gateway.description,
				location = listOfNotNull(gateway.city, gateway.region, country).joinToString(", "),
				countryCode = gateway.twoLetterCountryISO,
				mixnetScore = gateway.mixnetScore,
				score = gateway.wgScore,
				load = gateway.wgLoad,
				uptime = gateway.wgUptime?.let { it * 100 },
				lastUpdated = gateway.lastUpdated,
				asn = gateway.asn,
				asnName = gateway.asnName,
				asnKind = gateway.asnKind,
				buildVersion = gateway.buildVersion,
				exitIpv4 = gateway.exitIpv4s.firstOrNull(),
				exitIpv6 = gateway.exitIpv6s.firstOrNull(),
				isQuickSupportedByGateway = gateway.isQuicSupported(),
				nodeFamilyName = gateway.nodeFamilyName,
				isPostQuantumEnabled = gateway.isPostQuantumEnabled,
			)
		}
	}
}
