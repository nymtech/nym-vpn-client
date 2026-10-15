package net.nymtech.nymvpn.testutil

import net.nymtech.vpn.model.BridgeInformation
import net.nymtech.vpn.model.BridgeParameter
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.Score

internal fun gateway(
	identity: String,
	country: String? = "fr",
	name: String = identity,
	city: String? = null,
	region: String? = null,
	mixnetScore: Score? = Score.HIGH,
	wgScore: Score? = Score.HIGH,
) = NymGateway(
	identity = identity,
	twoLetterCountryISO = country,
	description = null,
	mixnetScore = mixnetScore,
	wgScore = wgScore,
	wgLoad = null,
	wgUptime = null,
	lastUpdated = null,
	name = name,
	city = city,
	region = region,
	asn = null,
	asnName = null,
	asnKind = null,
	buildVersion = null,
	exitIpv4s = emptyList(),
	exitIpv6s = emptyList(),
	bridgeInformation = null,
)

internal fun NymGateway.withQuic() = copy(bridgeInformation = BridgeInformation(listOf(BridgeParameter.QuicPlain())))
