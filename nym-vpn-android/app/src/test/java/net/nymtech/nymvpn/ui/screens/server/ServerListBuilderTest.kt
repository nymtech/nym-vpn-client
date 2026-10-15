package net.nymtech.nymvpn.ui.screens.server

import net.nymtech.nymvpn.testutil.gateway
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.FavoriteSelector
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.Locale

class ServerListBuilderTest {

	private val gateways = listOf(
		gateway("us-ca-1", country = "us", region = "California", city = "Los Angeles"),
		gateway("us-ca-2", country = "us", region = "California", city = "San Jose"),
		gateway("us-ny-1", country = "us", region = "New York"),
		gateway("us-free", country = "us"),
		gateway("fr-1", country = "fr", name = "paris-node", city = "Paris"),
		gateway("de-1", country = "de", name = "berlin-node"),
	)

	private val index = GatewayIndex.build(gateways, Tunnel.Mode.FIVE_HOP_MIXNET, false, setOf("us"), Locale.ENGLISH)

	private fun build(
		filter: ServerListFilter = ServerListFilter.ALL_SERVERS,
		query: String = "",
		favorites: List<FavoriteSelector> = emptyList(),
		recents: List<NymGateway> = emptyList(),
		selectedKey: String? = null,
		expansion: Map<String, Boolean> = emptyMap(),
	) = ServerListBuilder.build(
		index = index,
		filter = filter,
		query = query,
		favorites = favorites,
		recents = recents,
		selectedKey = selectedKey,
		userExpansion = expansion,
		locale = Locale.ENGLISH,
	)

	private fun List<ServerRow>.keys() = map { it.key }

	@Test
	fun collapsedCountriesProduceOnlyHeaders() {
		val rows = build().rows

		assertEquals(listOf("country:fr", "country:de", "country:us"), rows.keys())
		assertTrue(rows.all { it is ServerRow.Country && !it.expanded })
	}

	@Test
	fun expandedCountryFlattensRegionsAndUngroupedGateways() {
		val rows = build(expansion = mapOf("country:us" to true)).rows

		assertEquals(
			listOf("country:fr", "country:de", "country:us", "region:us/California", "region:us/New York", "gateway:us-free"),
			rows.keys(),
		)
		val usFree = rows.last() as ServerRow.Gateway
		assertTrue(usFree.roundBottom)
		assertFalse(usFree.standalone)
	}

	@Test
	fun expandedRegionListsItsGateways() {
		val rows = build(expansion = mapOf("country:us" to true, "region:us/New York" to true)).rows

		val keys = rows.keys()
		assertEquals(keys.indexOf("region:us/New York") + 1, keys.indexOf("gateway:us-ny-1"))
	}

	@Test
	fun lastRegionGatewayIsRoundedWhenNoUngroupedGateways() {
		val noFree = GatewayIndex.build(gateways.filter { it.identity != "us-free" }, Tunnel.Mode.FIVE_HOP_MIXNET, false, setOf("us"), Locale.ENGLISH)
		val rows = ServerListBuilder.build(noFree, ServerListFilter.ALL_SERVERS, "", emptyList(), emptyList(), null, mapOf("country:us" to true, "region:us/New York" to true), Locale.ENGLISH).rows

		val last = rows.last() as ServerRow.Gateway
		assertEquals("gateway:us-ny-1", last.key)
		assertTrue(last.roundBottom)
	}

	@Test
	fun countryContainingSelectionIsExpandedByDefault() {
		val rows = build(selectedKey = "us-ny-1").rows

		val us = rows.single { it.key == "country:us" } as ServerRow.Country
		assertTrue(us.expanded)
		assertTrue(us.containsSelection)
		val ny = rows.single { it.key == "region:us/New York" } as ServerRow.Region
		assertTrue(ny.expanded)
		val gateway = rows.single { it.key == "gateway:us-ny-1" } as ServerRow.Gateway
		assertTrue(gateway.isSelected)
	}

	@Test
	fun userCollapseOverridesDefaultExpansion() {
		val rows = build(selectedKey = "us-ny-1", expansion = mapOf("country:us" to false)).rows

		assertFalse((rows.single { it.key == "country:us" } as ServerRow.Country).expanded)
		assertFalse(rows.any { it.key.startsWith("region:") })
	}

	@Test
	fun selectedCountryIsMarkedSelected() {
		val rows = build(selectedKey = "fr").rows

		assertTrue((rows.single { it.key == "country:fr" } as ServerRow.Country).isSelected)
	}

	@Test
	fun queryMatchesCountryNameCodeAndRegion() {
		fun countries(query: String) = build(query = query).rows.filterIsInstance<ServerRow.Country>().map { it.code }

		assertEquals(listOf("fr"), countries("fra"))
		assertEquals(listOf("de"), countries("DE"))
		assertEquals(listOf("us"), countries("york"))
	}

	@Test
	fun queryMatchingRegionExpandsThatCountry() {
		val rows = build(query = "york").rows

		assertTrue((rows.first { it.key == "country:us" } as ServerRow.Country).expanded)
	}

	@Test
	fun queryAppendsDirectGatewayMatchesAsStandaloneRows() {
		val rows = build(query = "node").rows

		val standalone = rows.filterIsInstance<ServerRow.Gateway>().filter { it.standalone }
		assertEquals(listOf("gateway:fr-1", "gateway:de-1"), standalone.map { it.key })
		assertTrue(rows.none { it is ServerRow.Country })
	}

	@Test
	fun countsReflectEligiblePoolNotQuery() {
		val result = build(query = "fra")

		assertEquals(3, result.countryCount)
		assertEquals(6, result.nodeCount)
		assertFalse(result.isEmpty)
	}

	@Test
	fun favoritesFilterShowsOnlyFavoriteCountriesRegionsAndGateways() {
		val favorites = listOf(
			FavoriteSelector.Country("fr"),
			FavoriteSelector.Region("New York"),
			FavoriteSelector.Gateway("us-free"),
		)
		val result = build(filter = ServerListFilter.FAVORITES, favorites = favorites, expansion = mapOf("country:us" to true))

		assertEquals(listOf("country:fr", "country:us", "region:us/New York", "gateway:us-free"), result.rows.keys())
		assertTrue((result.rows[0] as ServerRow.Country).isFavorite)
		assertEquals(2, result.countryCount)
		assertEquals(3, result.nodeCount)
		assertEquals(setOf("us-free"), result.favoriteGatewayIds)
	}

	@Test
	fun favoritesFilterIsEmptyWithoutFavorites() {
		val result = build(filter = ServerListFilter.FAVORITES)

		assertTrue(result.rows.isEmpty())
		assertTrue(result.isEmpty)
	}

	@Test
	fun recentFilterListsRecentsAsStandaloneRows() {
		val recents = listOf(gateways[4], gateways[0])
		val result = build(filter = ServerListFilter.RECENT, recents = recents, favorites = listOf(FavoriteSelector.Gateway("fr-1")))

		assertEquals(listOf("gateway:fr-1", "gateway:us-ca-1"), result.rows.keys())
		assertTrue((result.rows[0] as ServerRow.Gateway).isFavorite)
		assertEquals("Paris, France", (result.rows[0] as ServerRow.Gateway).location)
		assertEquals("Los Angeles, California, United States", (result.rows[1] as ServerRow.Gateway).location)
		assertEquals(2, result.countryCount)
	}

	@Test
	fun recentFilterSearchesNameIdentityCityAndCountry() {
		val recents = listOf(gateways[4], gateways[0])

		assertEquals(listOf("gateway:fr-1"), build(filter = ServerListFilter.RECENT, recents = recents, query = "paris").rows.keys())
		assertEquals(listOf("gateway:us-ca-1"), build(filter = ServerListFilter.RECENT, recents = recents, query = "united").rows.keys())
	}

	@Test
	fun scrollTargetPointsAtRowContainingSelection() {
		val result = build(selectedKey = "de-1")

		assertEquals(result.rows.indexOfFirst { it.key == "country:de" }, result.scrollTargetIndex)
	}
}
