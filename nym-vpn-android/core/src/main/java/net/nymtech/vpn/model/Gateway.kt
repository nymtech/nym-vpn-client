package net.nymtech.vpn.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import nym_vpn_lib.Gateway
import nym_vpn_lib.NodeIdentity
import nym_vpn_lib.Percent


@Serializable
data class NymGateway(
	val identity: NodeIdentity,
	val twoLetterCountryISO: String?,
	val mixnetPerformance: Percent?,
	val probeResult: ProbeResult?
) {
	companion object {
		fun from(gateway: Gateway) : NymGateway {
			return NymGateway(
				identity = gateway.identity,
				twoLetterCountryISO = gateway.location?.twoLetterIsoCountryCode?.lowercase(),
				mixnetPerformance = gateway.mixnetPerformance,
				probeResult = gateway.lastProbe?.let {
					ProbeResult(
						lastUpdatedUtc = it.lastUpdatedUtc,
						entryCanRoute = it.outcome.asEntry.canRoute,
						entryCanConnect = it.outcome.asEntry.canConnect,
						exitCanConnect = it.outcome.asExit?.canConnect,
						exitCanRouteIpV4 = it.outcome.asExit?.canRouteIpV4,
						exitCanRouteIpV6 = it.outcome.asExit?.canRouteIpV6,
						exitCanRouteIpExternalV4 = it.outcome.asExit?.canRouteIpExternalV4,
						exitCanRouteIpExternalV6 = it.outcome.asExit?.canRouteIpExternalV6,
						wgProbeResult = it.outcome.wg?.let {
							WgProbeResult(
								canRegister = it.canRegister,
								canHandshake = it.canHandshake,
								canResolveDns = it.canResolveDns,
								pingHostsPerformance = it.pingHostsPerformance,
								pingIpsPerformance = it.pingHostsPerformance
							)
						}
					)
				}
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

@Serializable
data class ProbeResult (
	val lastUpdatedUtc: String?,
	val entryCanConnect: Boolean?,
	val entryCanRoute: Boolean,
	var exitCanConnect: Boolean?,
	val exitCanRouteIpV4: Boolean?,
	val exitCanRouteIpExternalV4: Boolean?,
	var exitCanRouteIpV6: Boolean?,
	var exitCanRouteIpExternalV6: Boolean?,
	var wgProbeResult: WgProbeResult?
)

@Serializable
data class WgProbeResult (
	val canRegister: Boolean,
	var canHandshake: Boolean,
	var canResolveDns: Boolean,
	var pingHostsPerformance: Float,
	var pingIpsPerformance: Float
)
