package net.nymtech.vpn.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import nym_vpn_lib_types.AsnKind
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.GatewayInfo
import nym_vpn_lib_types.Ipv4Addr
import nym_vpn_lib_types.Ipv6Addr
import nym_vpn_lib_types.NodeIdentity
import nym_vpn_lib_types.Score

@Serializable
data class NymGateway(
	val identity: NodeIdentity,
	val twoLetterCountryISO: String?,
	val mixnetScore: Score?,
	val wgScore: Score?,
	val wgLoad: Score?,
	val wgUptime: Float?,
	val lastUpdated: String?,
	val name: String,
	val city: String?,
	val region: String?,
	val asn: String?,
	val asnName: String?,
	val asnKind: AsnKind?,
	val buildVersion: String?,
	var exitIpv4s: List<Ipv4Addr>,
	var exitIpv6s: List<Ipv6Addr>,
) {
	companion object {
		fun from(gateway: GatewayInfo): NymGateway {
			return NymGateway(
				identity = gateway.id,
				name = gateway.moniker,
				twoLetterCountryISO = gateway.location?.twoLetterIsoCountryCode?.lowercase(),
				mixnetScore = gateway.mixnetScore,
				wgScore = gateway.wgPerformance?.score,
				wgLoad = gateway.wgPerformance?.load,
				lastUpdated = gateway.wgPerformance?.lastUpdatedUtc,
				wgUptime = gateway.wgPerformance?.uptimePercentageLast24Hours,
				city = gateway.location?.city,
				region = gateway.location?.region,
				asn = gateway.location?.asn?.asn,
				asnName = gateway.location?.asn?.name,
				asnKind = gateway.location?.asn?.kind,
				buildVersion = gateway.buildVersion,
				exitIpv4s = gateway.exitIpv4s,
				exitIpv6s = gateway.exitIpv6s,
			)
		}

		fun from(string: String?): NymGateway? {
			return string?.let { Json.decodeFromString<NymGateway>(string) }
		}

		fun fromCollectionString(string: String?): List<NymGateway> {
			return string?.let {
				Json.decodeFromString<List<NymGateway>>(it)
			} ?: emptyList()
		}
	}
	override fun toString(): String {
		return Json.encodeToString(serializer(), this)
	}

	fun toLocationEntryPoint(): EntryPoint? {
		return twoLetterCountryISO?.let {
			EntryPoint.Location(twoLetterCountryISO)
		}
	}

	fun toLocationExitPoint(): ExitPoint? {
		return twoLetterCountryISO?.let {
			ExitPoint.Location(twoLetterCountryISO)
		}
	}

	fun toGatewayEntryPoint(): EntryPoint? {
		return EntryPoint.Gateway(identity)
	}

	fun toGatewayExitPoint(): ExitPoint {
		return ExitPoint.Gateway(identity)
	}
}
