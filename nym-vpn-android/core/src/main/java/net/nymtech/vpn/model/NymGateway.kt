package net.nymtech.vpn.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import nym_vpn_lib_types.AsnKind
import nym_vpn_lib_types.Gateway
import nym_vpn_lib_types.Ipv4Addr
import nym_vpn_lib_types.Ipv6Addr
import nym_vpn_lib_types.NodeIdentity
import nym_vpn_lib_types.Score
import nym_vpn_lib_types.BridgeInformation as SdkBridgeInformation
import nym_vpn_lib_types.BridgeParameters as SdkBridgeParameter

/**
 * Gateway domain model mapped from SDK types.
 */
@Serializable
data class NymGateway(
	val identity: NodeIdentity,
	val twoLetterCountryISO: String?,
	val description: String?,
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
	val bridgeInformation: BridgeInformation?,
	val nodeFamilyName: String?,
	val isPostQuantumEnabled: Boolean,
) {
	companion object {
		fun from(gateway: Gateway): NymGateway = NymGateway(
			identity = gateway.identityKey,
			name = gateway.name,
			description = gateway.description,
			twoLetterCountryISO = gateway.location?.twoLetterIsoCountryCode?.lowercase(),
			mixnetScore = gateway.performance?.mixnetScore,
			wgScore = gateway.performance?.score,
			wgLoad = gateway.performance?.load,
			lastUpdated = gateway.performance?.lastUpdatedUtc,
			wgUptime = gateway.performance?.uptimePercentageLast24Hours,
			city = gateway.location?.city,
			region = gateway.location?.region,
			asn = gateway.location?.asn?.asn,
			asnName = gateway.location?.asn?.name,
			asnKind = gateway.location?.asn?.kind,
			buildVersion = gateway.buildVersion,
			exitIpv4s = gateway.exitIpv4s,
			exitIpv6s = gateway.exitIpv6s,
			bridgeInformation = gateway.bridgeParams?.toBridgeInformation(),
			nodeFamilyName = gateway.nodeFamilyName,
			isPostQuantumEnabled = gateway.lewesProtocolDetails?.content?.enabled == true,
		)

		fun from(string: String?): NymGateway? = string?.let { Json.decodeFromString<NymGateway>(string) }

		fun fromCollectionString(string: String?): List<NymGateway> = string?.let {
			Json.decodeFromString<List<NymGateway>>(it)
		} ?: emptyList()

		private fun SdkBridgeInformation.toBridgeInformation() = BridgeInformation(
			transports = transports.map {
				it.toBridgeParameter()
			},
		)

		private fun SdkBridgeParameter.toBridgeParameter() = when (this) {
			is SdkBridgeParameter.QuicPlain -> BridgeParameter.QuicPlain()
		}
	}

	override fun toString(): String = Json.encodeToString(serializer(), this)
}

@Serializable
data class BridgeInformation(val transports: List<BridgeParameter>)

@Serializable
sealed class BridgeParameter {
	@Serializable
	class QuicPlain : BridgeParameter()
}
