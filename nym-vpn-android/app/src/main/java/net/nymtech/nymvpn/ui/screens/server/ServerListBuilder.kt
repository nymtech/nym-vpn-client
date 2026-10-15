package net.nymtech.nymvpn.ui.screens.server

import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.FavoriteSelector
import java.util.Locale

/** Pure function from a [GatewayIndex] plus UI state to the flat row list. Safe to run on any dispatcher. */
object ServerListBuilder {

	fun build(
		index: GatewayIndex,
		filter: ServerListFilter,
		query: String,
		favorites: List<FavoriteSelector>,
		recents: List<NymGateway>,
		selectedKey: String?,
		/** Explicit user expand/collapse choices by row key; rows absent here use their default expansion. */
		userExpansion: Map<String, Boolean>,
		locale: Locale = Locale.getDefault(),
	): ServerListResult {
		val ctx = Context(index, query.trim().lowercase(locale), Favorites.from(favorites), selectedKey, userExpansion, locale)
		return when (filter) {
			ServerListFilter.ALL_SERVERS -> buildAll(ctx)
			ServerListFilter.FAVORITES -> buildFavorites(ctx)
			ServerListFilter.RECENT -> buildRecents(ctx, recents)
		}
	}

	private class Favorites(val countries: Set<String>, val regions: Set<String>, val gatewayIds: Set<String>) {
		companion object {
			fun from(selectors: List<FavoriteSelector>) = Favorites(
				countries = selectors.filterIsInstance<FavoriteSelector.Country>().map { it.twoLetterIsoCountryCode.lowercase() }.toSet(),
				regions = selectors.filterIsInstance<FavoriteSelector.Region>().map { it.region.lowercase() }.toSet(),
				gatewayIds = selectors.filterIsInstance<FavoriteSelector.Gateway>().map { it.identity }.toSet(),
			)
		}
	}

	private class Context(val index: GatewayIndex, val query: String, val favorites: Favorites, val selectedKey: String?, val userExpansion: Map<String, Boolean>, val locale: Locale) {
		val hasQuery get() = query.isNotEmpty()

		fun matchesCountry(country: GatewayIndex.CountryEntry) = !hasQuery ||
			country.searchTerms.any { it.contains(query) } ||
			country.regionTerms.any { it.contains(query) }

		fun matchesGateway(gateway: NymGateway) = gateway.identity.lowercase().contains(query) || gateway.name.lowercase().contains(query)

		fun isSelected(gateway: NymGateway) = gateway.identity == selectedKey

		fun selectionInside(gateways: List<NymGateway>) = selectedKey != null &&
			gateways.any { it.identity == selectedKey || it.region.equals(selectedKey, ignoreCase = true) }

		fun expanded(key: String, default: Boolean) = userExpansion[key] ?: default

		fun displayCountry(code: String?): String? = code?.let { index.countriesByCode[it.lowercase()]?.displayName ?: Locale("", it).getDisplayCountry(locale) }

		fun location(gateway: NymGateway): String {
			val region = gateway.region.takeIf { gateway.twoLetterCountryISO?.lowercase() in index.regionCountryCodes }
			return listOfNotNull(gateway.city, region, displayCountry(gateway.twoLetterCountryISO)).joinToString(", ")
		}
	}

	private fun buildAll(ctx: Context): ServerListResult {
		val rows = mutableListOf<ServerRow>()
		ctx.index.countries.filter { ctx.matchesCountry(it) }.forEach { appendCountry(rows, ctx, it, it.regions, it.ungrouped, it.code in ctx.favorites.countries) }
		if (ctx.hasQuery) appendStandalone(rows, ctx, ctx.index.gateways.filter { ctx.matchesGateway(it) })
		return result(rows, ctx, ctx.index.countryCount, ctx.index.nodeCount, ctx.index.nodeCount == 0)
	}

	private fun buildFavorites(ctx: Context): ServerListResult {
		val fav = ctx.favorites
		val pruned = ctx.index.countries.mapNotNull { country ->
			val countryFavorite = country.code in fav.countries
			val regions = country.regions?.mapNotNull { region ->
				val regionFavorite = region.name.lowercase() in fav.regions
				val kept = if (countryFavorite || regionFavorite) region.gateways else region.gateways.filter { it.identity in fav.gatewayIds }
				if (kept.isEmpty()) null else GatewayIndex.RegionEntry(region.name, kept)
			}?.takeIf { it.isNotEmpty() }
			val ungrouped = if (countryFavorite) country.ungrouped else country.ungrouped.filter { it.identity in fav.gatewayIds }
			if (regions == null && ungrouped.isEmpty()) null else PrunedCountry(country, regions, ungrouped, countryFavorite)
		}
		val favoriteNodeIds = pruned.flatMap { p -> (p.regions.orEmpty().flatMap { it.gateways } + p.ungrouped).map { it.identity } }.toSet()

		val rows = mutableListOf<ServerRow>()
		pruned.filter { ctx.matchesCountry(it.entry) }.forEach { appendCountry(rows, ctx, it.entry, it.regions, it.ungrouped, it.isFavorite) }
		if (ctx.hasQuery) appendStandalone(rows, ctx, ctx.index.gateways.filter { it.identity in favoriteNodeIds && ctx.matchesGateway(it) })
		return result(rows, ctx, pruned.size, favoriteNodeIds.size, pruned.isEmpty())
	}

	private class PrunedCountry(val entry: GatewayIndex.CountryEntry, val regions: List<GatewayIndex.RegionEntry>?, val ungrouped: List<NymGateway>, val isFavorite: Boolean)

	private fun buildRecents(ctx: Context, recents: List<NymGateway>): ServerListResult {
		val matching = recents.filter { gateway ->
			!ctx.hasQuery ||
				ctx.matchesGateway(gateway) ||
				gateway.city?.lowercase()?.contains(ctx.query) == true ||
				ctx.displayCountry(gateway.twoLetterCountryISO)?.lowercase(ctx.locale)?.contains(ctx.query) == true
		}
		val rows = mutableListOf<ServerRow>()
		appendStandalone(rows, ctx, matching)
		val countryCount = recents.mapNotNull { it.twoLetterCountryISO }.distinct().size
		return result(rows, ctx, countryCount, recents.size, recents.isEmpty())
	}

	private fun appendCountry(
		rows: MutableList<ServerRow>,
		ctx: Context,
		country: GatewayIndex.CountryEntry,
		regions: List<GatewayIndex.RegionEntry>?,
		ungrouped: List<NymGateway>,
		isFavorite: Boolean,
	) {
		val shownGateways = regions.orEmpty().flatMap { it.gateways } + ungrouped
		val containsSelection = ctx.selectionInside(shownGateways)
		val queryHitsRegion = ctx.hasQuery && country.regionTerms.any { it.contains(ctx.query) }
		val countryRow = ServerRow.Country(
			code = country.code,
			displayName = country.displayName,
			nodeCount = shownGateways.size,
			isFavorite = isFavorite,
			isSelected = country.code == ctx.selectedKey,
			containsSelection = containsSelection,
			expanded = ctx.expanded("country:${country.code}", containsSelection || queryHitsRegion),
		)
		rows.add(countryRow)
		if (!countryRow.expanded) return

		val regionsAreLastBlock = ungrouped.isEmpty()
		regions?.forEachIndexed { index, region ->
			val isLastRegion = regionsAreLastBlock && index == regions.lastIndex
			val regionRow = ServerRow.Region(
				countryCode = country.code,
				name = region.name,
				nodeCount = region.gateways.size,
				isFavorite = region.name.lowercase() in ctx.favorites.regions,
				isSelected = region.name.equals(ctx.selectedKey, ignoreCase = true),
				expanded = ctx.expanded("region:${country.code}/${region.name}", ctx.selectionInside(region.gateways)),
				roundBottom = false,
			).let { it.copy(roundBottom = isLastRegion && !it.expanded) }
			rows.add(regionRow)
			if (regionRow.expanded) {
				region.gateways.forEachIndexed { gatewayIndex, gateway ->
					rows.add(blockGateway(ctx, gateway, roundBottom = isLastRegion && gatewayIndex == region.gateways.lastIndex))
				}
			}
		}
		ungrouped.forEachIndexed { index, gateway ->
			rows.add(blockGateway(ctx, gateway, roundBottom = index == ungrouped.lastIndex))
		}
	}

	private fun blockGateway(ctx: Context, gateway: NymGateway, roundBottom: Boolean) = ServerRow.Gateway(
		gateway = gateway,
		isFavorite = gateway.identity in ctx.favorites.gatewayIds,
		isSelected = ctx.isSelected(gateway),
		roundBottom = roundBottom,
		standalone = false,
		location = null,
	)

	private fun appendStandalone(rows: MutableList<ServerRow>, ctx: Context, gateways: List<NymGateway>) {
		// A gateway already visible inside an expanded country block must not get a second row: LazyColumn keys are unique.
		val shown = rows.filterIsInstance<ServerRow.Gateway>().mapTo(HashSet()) { it.gateway.identity }
		gateways.distinctBy { it.identity }.filter { it.identity !in shown }.forEach { gateway ->
			rows.add(
				ServerRow.Gateway(
					gateway = gateway,
					isFavorite = gateway.identity in ctx.favorites.gatewayIds,
					isSelected = ctx.isSelected(gateway),
					roundBottom = true,
					standalone = true,
					location = ctx.location(gateway),
				),
			)
		}
	}

	private fun result(rows: List<ServerRow>, ctx: Context, countryCount: Int, nodeCount: Int, isEmpty: Boolean) = ServerListResult(
		rows = rows,
		countryCount = countryCount,
		nodeCount = nodeCount,
		isEmpty = isEmpty,
		favoriteGatewayIds = ctx.favorites.gatewayIds,
		scrollTargetIndex = rows.indexOfFirst { row ->
			when (row) {
				is ServerRow.Country -> row.isSelected || row.containsSelection
				is ServerRow.Region -> row.isSelected
				is ServerRow.Gateway -> row.isSelected
			}
		},
	)
}
