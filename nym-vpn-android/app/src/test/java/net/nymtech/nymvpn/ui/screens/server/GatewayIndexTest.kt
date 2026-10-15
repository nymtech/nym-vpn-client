package net.nymtech.nymvpn.ui.screens.server

import net.nymtech.nymvpn.testutil.gateway
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.Score
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.util.Locale

class GatewayIndexTest {

	private val regionCountries = setOf("us", "ca")

	private fun build(vararg gateways: net.nymtech.vpn.model.NymGateway, mode: Tunnel.Mode = Tunnel.Mode.FIVE_HOP_MIXNET, quicOnly: Boolean = false) =
		GatewayIndex.build(gateways.toList(), mode, quicOnly, regionCountries, Locale.ENGLISH)

	@Test
	fun countriesSortedByDisplayNameAndGatewaysByScore() {
		val index = build(
			gateway("fr-low", country = "fr", mixnetScore = Score.LOW),
			gateway("de-1", country = "de"),
			gateway("fr-high", country = "fr", mixnetScore = Score.HIGH),
		)

		assertEquals(listOf("fr", "de"), index.countries.map { it.code })
		assertEquals(listOf("France", "Germany"), index.countries.map { it.displayName })
		assertEquals(listOf("fr-high", "fr-low"), index.countries.first().gateways.map { it.identity })
	}

	@Test
	fun gatewaysWithoutCountryAreExcludedFromCountriesButCounted() {
		val index = build(gateway("nowhere", country = null), gateway("fr-1"))

		assertEquals(1, index.countries.size)
		assertEquals(1, index.countryCount)
		assertEquals(2, index.nodeCount)
	}

	@Test
	fun quicOnlyFilterDropsGatewaysWithoutQuic() {
		val index = build(gateway("plain"), gateway("quic", quic = true), quicOnly = true)

		assertEquals(listOf("quic"), index.gateways.map { it.identity })
		assertEquals(1, index.nodeCount)
	}

	@Test
	fun regionsGroupedOnlyForSupportedCountriesWithMultipleRegions() {
		val index = build(
			gateway("us-ca", country = "us", region = "California"),
			gateway("us-ny", country = "us", region = "New York"),
			gateway("us-none", country = "us", region = null),
			gateway("de-he", country = "de", region = "Hesse"),
			gateway("de-by", country = "de", region = "Bavaria"),
			gateway("ca-on1", country = "ca", region = "Ontario"),
			gateway("ca-on2", country = "ca", region = "Ontario"),
		)

		val us = index.countries.single { it.code == "us" }
		assertEquals(listOf("California", "New York"), us.regions?.map { it.name })
		assertEquals(listOf("us-none"), us.ungrouped.map { it.identity })
		assertNull(index.countries.single { it.code == "de" }.regions)
		assertNull(index.countries.single { it.code == "ca" }.regions)
	}

	@Test
	fun searchTermsIncludeDisplayNameCodeAndIso3() {
		val index = build(gateway("fr-1"))

		val terms = index.countries.single().searchTerms
		assertEquals(true, "france" in terms)
		assertEquals(true, "fr" in terms)
		assertEquals(true, "fra" in terms)
	}

	@Test
	fun twoHopModeSortsByWgScore() {
		val index = build(
			gateway("a", wgScore = Score.LOW, mixnetScore = Score.HIGH),
			gateway("b", wgScore = Score.HIGH, mixnetScore = Score.LOW),
			mode = Tunnel.Mode.TWO_HOP_MIXNET,
		)

		assertEquals(listOf("b", "a"), index.countries.single().gateways.map { it.identity })
	}
}
