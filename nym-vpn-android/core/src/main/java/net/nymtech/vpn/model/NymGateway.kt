package net.nymtech.vpn.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import nym_vpn_lib.GatewayInfo
import nym_vpn_lib.NodeIdentity
import nym_vpn_lib.Score

@Serializable
data class NymGateway(
	val identity: NodeIdentity,
	val twoLetterCountryISO: String?,
	val mixnetScore: Score?,
	val wgScore: Score?,
	val name: String,
) {
	companion object {
		fun from(gateway: GatewayInfo): NymGateway {
			return NymGateway(
				identity = gateway.id,
				name = gateway.moniker,
				twoLetterCountryISO = gateway.location?.twoLetterIsoCountryCode?.lowercase(),
				mixnetScore = gateway.mixnetScore,
				wgScore = gateway.wgScore,
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
