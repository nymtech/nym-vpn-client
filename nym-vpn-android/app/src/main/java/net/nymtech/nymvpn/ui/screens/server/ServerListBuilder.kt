// Builds server list items from a gateway snapshot
package net.nymtech.nymvpn.ui.screens.server

import net.nymtech.nymvpn.util.Constants.countryCodesForRegionSupport
import net.nymtech.nymvpn.util.extensions.isQuicSupported
import net.nymtech.nymvpn.util.extensions.iso3CountryOrEmpty
import net.nymtech.nymvpn.util.extensions.scoreSorted
import net.nymtech.nymvpn.util.extensions.toLocale
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.util.extensions.toDisplayCountry
import nym_vpn_lib_types.FavoriteSelector
import java.text.Collator
import java.util.Locale

internal data class ServerListInput(
	val filter: ServerListFilter,
	val query: String,
	val gateways: List<NymGateway>,
	val recentGateways: List<NymGateway>,
	val selectors: List<FavoriteSelector>,
	val isQuicOnly: Boolean,
	val tunnelMode: Tunnel.Mode,
)

internal data class ServerListResult(val items: List<ItemType>, val countryCount: Int, val nodeCount: Int, val isEmpty: Boolean, val favoriteGatewayIds: Set<String>)

internal class GatewayIndex private constructor(
	private val source: List<NymGateway>,
	private val isQuicOnly: Boolean,
	private val tunnelMode: Tunnel.Mode,
	private val locale: Locale,
	val eligible: List<NymGateway>,
	val countries: List<Country>,
	val countriesByCode: Map<String, Country>,
	val countryCount: Int,
) {
	class Country(val code: String, val locale: Locale, val gateways: List<NymGateway>) {
		private val displayCountry = locale.displayCountry.lowercase()
		private val iso3 = locale.iso3CountryOrEmpty().lowercase()
		private val regions = gateways.mapNotNullTo(HashSet()) { it.region?.lowercase() }

		fun matches(lowercaseQuery: String): Boolean = displayCountry.contains(lowercaseQuery) ||
			code.contains(lowercaseQuery) ||
			iso3.contains(lowercaseQuery) ||
			regions.any { it.contains(lowercaseQuery) }
	}

	fun isFor(input: ServerListInput): Boolean = source === input.gateways && isQuicOnly == input.isQuicOnly && tunnelMode == input.tunnelMode && locale == Locale.getDefault()

	companion object {
		fun build(input: ServerListInput): GatewayIndex {
			// Stable sort keeps score order within country groups and search results
			val eligible = input.gateways
				.filter { !input.isQuicOnly || it.isQuicSupported() }
				.scoreSorted(input.tunnelMode)
			val countriesByCode = eligible
				.filter { it.twoLetterCountryISO != null }
				.groupBy { it.twoLetterCountryISO!!.lowercase() }
				.mapNotNull { (code, gateways) -> gateways.first().toLocale()?.let { code to Country(code, it, gateways) } }
				.toMap()
			val collator = Collator.getInstance()
			val countries = countriesByCode.values
				.distinctBy { it.locale.displayCountry }
				.sortedWith(compareBy(collator) { it.locale.displayCountry })
			return GatewayIndex(
				source = input.gateways,
				isQuicOnly = input.isQuicOnly,
				tunnelMode = input.tunnelMode,
				locale = Locale.getDefault(),
				eligible = eligible,
				countries = countries,
				countriesByCode = countriesByCode,
				countryCount = eligible.mapNotNull { it.twoLetterCountryISO }.distinct().size,
			)
		}
	}
}

internal fun buildServerList(input: ServerListInput, index: GatewayIndex): ServerListResult = when (input.filter) {
	ServerListFilter.ALL_SERVERS -> buildAllServers(input, index)
	ServerListFilter.FAVORITES -> buildFavorites(input, index)
	ServerListFilter.RECENT -> buildRecent(input)
}

private fun buildAllServers(input: ServerListInput, index: GatewayIndex): ServerListResult {
	val lowercaseQuery = input.query.lowercase()
	val selectors = input.selectors

	val items = mutableListOf<ItemType>()
	index.countries
		.filter { it.matches(lowercaseQuery) }
		.mapTo(items) { createCountryItem(it.locale, it.gateways, selectors) }

	if (input.query.isNotBlank()) {
		index.eligible
			.filter { it.matchesGatewayQuery(lowercaseQuery) }
			.distinctBy { it.identity }
			.mapTo(items) { ItemType.GatewayItem(it, isGatewayFavorite(it.identity, selectors)) }
	}

	return ServerListResult(
		items = items,
		countryCount = index.countryCount,
		nodeCount = index.eligible.size,
		isEmpty = index.eligible.isEmpty(),
		favoriteGatewayIds = favoriteGatewayIds(selectors),
	)
}

private fun buildFavorites(input: ServerListInput, index: GatewayIndex): ServerListResult {
	val lowercaseQuery = input.query.lowercase()
	val selectors = input.selectors

	val favoriteGatewayIds = favoriteGatewayIds(selectors)
	val favoriteRegions = selectors.filterIsInstance<FavoriteSelector.Region>().map { it.region.lowercase() }.toSet()
	val favoriteCountries = selectors.filterIsInstance<FavoriteSelector.Country>().map { it.twoLetterIsoCountryCode.lowercase() }.toSet()

	val relevantCountryCodes = favoriteCountries + index.countriesByCode.filterValues { country ->
		country.gateways.any { it.identity in favoriteGatewayIds || (it.region != null && it.region!!.lowercase() in favoriteRegions) }
	}.keys

	val allFavoriteItems = relevantCountryCodes.mapNotNull { code ->
		val country = index.countriesByCode[code] ?: return@mapNotNull null
		val full = createCountryItem(country.locale, country.gateways, selectors)
		pruneToFavorites(full, code in favoriteCountries, favoriteRegions, favoriteGatewayIds)
	}.filter { it.gateways.isNotEmpty() || !it.regions.isNullOrEmpty() }

	val allNodeIds = mutableSetOf<String>()
	allFavoriteItems.forEach { item ->
		val grouped = item.regions?.flatMap { it.gateways }.orEmpty()
		val ungrouped = if (item.regions != null) item.gateways.filter { it.region == null } else item.gateways
		allNodeIds += (grouped + ungrouped).map { it.identity }
	}

	val collator = Collator.getInstance()
	val items = mutableListOf<ItemType>()
	allFavoriteItems
		.filter { item ->
			input.query.isBlank() ||
				item.locale.displayCountry.lowercase().contains(lowercaseQuery) ||
				item.locale.country.lowercase().contains(lowercaseQuery) ||
				item.locale.iso3CountryOrEmpty().lowercase().contains(lowercaseQuery) ||
				item.regions?.any { it.region.lowercase().contains(lowercaseQuery) } == true
		}
		.sortedWith(compareBy(collator) { it.locale.displayCountry })
		.let(items::addAll)

	if (input.query.isNotBlank()) {
		index.eligible
			.filter { it.identity in allNodeIds && it.matchesGatewayQuery(lowercaseQuery) }
			.distinctBy { it.identity }
			.mapTo(items) { ItemType.GatewayItem(it, it.identity in favoriteGatewayIds) }
	}

	return ServerListResult(
		items = items,
		countryCount = allFavoriteItems.size,
		nodeCount = allNodeIds.size,
		isEmpty = allFavoriteItems.isEmpty(),
		favoriteGatewayIds = favoriteGatewayIds,
	)
}

private fun buildRecent(input: ServerListInput): ServerListResult {
	val lowercaseQuery = input.query.lowercase()
	val selectors = input.selectors
	val recent = input.recentGateways

	val items = recent
		.filter { gateway ->
			input.query.isBlank() ||
				gateway.name.lowercase().contains(lowercaseQuery) ||
				gateway.identity.lowercase().contains(lowercaseQuery) ||
				gateway.city?.lowercase()?.contains(lowercaseQuery) == true ||
				gateway.twoLetterCountryISO?.let { toDisplayCountry(it).lowercase().contains(lowercaseQuery) } == true
		}
		.map { ItemType.GatewayItem(it, isGatewayFavorite(it.identity, selectors)) }

	return ServerListResult(
		items = items,
		countryCount = recent.mapNotNull { it.twoLetterCountryISO }.distinct().size,
		nodeCount = recent.size,
		isEmpty = recent.isEmpty(),
		favoriteGatewayIds = favoriteGatewayIds(selectors),
	)
}

private fun NymGateway.matchesGatewayQuery(lowercaseQuery: String): Boolean = identity.lowercase().contains(lowercaseQuery) || name.lowercase().contains(lowercaseQuery)

private fun createCountryItem(locale: Locale, gateways: List<NymGateway>, selectors: List<FavoriteSelector>): ItemType.CountryItem {
	val countryCode = locale.country
	val regions = if (countryCode.lowercase() in countryCodesForRegionSupport) {
		gateways.filter { it.region != null }
			.groupBy { it.region }
			.mapNotNull { (region, regionGateways) ->
				if (region == null) return@mapNotNull null
				ItemType.CountryItem.Region(region, regionGateways, isRegionFavorite(region, selectors))
			}
			.sortedBy { it.region }
	} else {
		null
	}
	return ItemType.CountryItem(locale, gateways, regions, isCountryFavorite(countryCode, selectors))
}

private fun pruneToFavorites(item: ItemType.CountryItem, isCountryFavorite: Boolean, favoriteRegions: Set<String>, favoriteGatewayIds: Set<String>): ItemType.CountryItem {
	val regions = item.regions?.mapNotNull { region ->
		val isRegionFavorite = region.region.lowercase() in favoriteRegions
		val keepGateways = if (isCountryFavorite || isRegionFavorite) {
			region.gateways
		} else {
			region.gateways.filter { it.identity in favoriteGatewayIds }
		}
		if (!isCountryFavorite && !isRegionFavorite && keepGateways.isEmpty()) return@mapNotNull null
		region.copy(gateways = keepGateways, isFavorite = isRegionFavorite)
	}?.takeIf { it.isNotEmpty() }

	val gateways = if (isCountryFavorite) {
		item.gateways
	} else {
		item.gateways.filter { it.identity in favoriteGatewayIds || it.region?.lowercase() in favoriteRegions }
	}

	return item.copy(gateways = gateways, regions = regions, isFavorite = isCountryFavorite)
}

private fun isCountryFavorite(twoLetterCountryISO: String, selectors: List<FavoriteSelector>): Boolean =
	selectors.any { it is FavoriteSelector.Country && it.twoLetterIsoCountryCode.equals(twoLetterCountryISO, true) }

private fun isRegionFavorite(region: String, selectors: List<FavoriteSelector>): Boolean = selectors.any { it is FavoriteSelector.Region && it.region.equals(region, true) }

private fun isGatewayFavorite(identity: String, selectors: List<FavoriteSelector>): Boolean = selectors.any { it is FavoriteSelector.Gateway && it.identity == identity }

private fun favoriteGatewayIds(selectors: List<FavoriteSelector>): Set<String> = selectors.filterIsInstance<FavoriteSelector.Gateway>().map { it.identity }.toSet()
