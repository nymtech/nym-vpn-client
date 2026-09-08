package net.nymtech.nymvpn.util.extensions

import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.Score
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class ScoreForModeTest {

	private fun gateway(mixnetScore: Score?, wgScore: Score?) = NymGateway(
		identity = "id",
		twoLetterCountryISO = "fr",
		description = null,
		mixnetScore = mixnetScore,
		wgScore = wgScore,
		wgLoad = null,
		wgUptime = null,
		lastUpdated = null,
		name = "gateway",
		city = null,
		region = null,
		asn = null,
		asnName = null,
		asnKind = null,
		buildVersion = null,
		exitIpv4s = emptyList(),
		exitIpv6s = emptyList(),
		bridgeInformation = null,
	)

	@Test
	fun fiveHopModeUsesMixnetScore() {
		val gateway = gateway(mixnetScore = Score.LOW, wgScore = Score.HIGH)
		assertEquals(Score.LOW, gateway.scoreFor(Tunnel.Mode.FIVE_HOP_MIXNET))
	}

	@Test
	fun twoHopModeUsesWgScore() {
		val gateway = gateway(mixnetScore = Score.HIGH, wgScore = Score.LOW)
		assertEquals(Score.LOW, gateway.scoreFor(Tunnel.Mode.TWO_HOP_MIXNET))
	}

	@Test
	fun missingScoreIsNullNotHigh() {
		val gateway = gateway(mixnetScore = null, wgScore = null)
		assertNull(gateway.scoreFor(Tunnel.Mode.FIVE_HOP_MIXNET))
		assertNull(gateway.scoreFor(Tunnel.Mode.TWO_HOP_MIXNET))
	}
}
