// Builds the flat list of rows rendered by the server screen from grouped gateways.
package net.nymtech.nymvpn.ui.screens.server

import net.nymtech.vpn.model.NymGateway

internal fun buildServerRows(
	items: List<ItemType>,
	includeAutoOptions: Boolean,
	query: String,
	selectedKey: String?,
	favoriteGatewayIds: Set<String>,
	expandStatePrefix: String,
	expandedOverrides: Map<String, Boolean>,
): List<ServerRow> = buildList {
	if (includeAutoOptions) {
		add(ServerRow.Safest)
		add(ServerRow.Random)
	}
	items.forEach { item ->
		when (item) {
			is ItemType.CountryItem -> addCountry(item, query, selectedKey, favoriteGatewayIds, expandStatePrefix, expandedOverrides)
			is ItemType.GatewayItem -> add(ServerRow.SearchResult(item.gateway, item.isFavorite))
		}
	}
}

private fun MutableList<ServerRow>.addCountry(
	item: ItemType.CountryItem,
	query: String,
	selectedKey: String?,
	favoriteGatewayIds: Set<String>,
	expandStatePrefix: String,
	expandedOverrides: Map<String, Boolean>,
) {
	val stateKey = "${expandStatePrefix}_country_${item.locale.country}"
	val expanded = expandedOverrides[stateKey] ?: item.isExpandedByDefault(query, selectedKey)
	val header = ServerRow.CountryHeader(
		locale = item.locale,
		gatewayCount = item.gateways.size,
		isFavorite = item.isFavorite,
		isExpanded = expanded,
		expandStateKey = stateKey,
	)
	add(header)
	if (!expanded) return

	val regions = item.regions.orEmpty()
	val ungrouped = (if (item.regions != null) item.gateways.filter { it.region == null } else item.gateways).uniqueRows()

	regions.forEachIndexed { index, region ->
		val regionStateKey = "${expandStatePrefix}_region_${region.region}"
		val regionExpanded = expandedOverrides[regionStateKey]
			?: region.gateways.any { it.region.equals(selectedKey, true) || it.identity == selectedKey }
		val isLastRegion = ungrouped.isEmpty() && index == regions.lastIndex
		val regionHeader = ServerRow.RegionHeader(
			locale = item.locale,
			region = region.region,
			gatewayCount = region.gateways.size,
			isFavorite = region.isFavorite,
			isExpanded = regionExpanded,
			expandStateKey = regionStateKey,
			isLastInGroup = isLastRegion && !regionExpanded,
		)
		add(regionHeader)
		if (regionExpanded) addGateways(regionHeader.key, region.gateways.uniqueRows(), isLastRegion, favoriteGatewayIds)
	}
	addGateways(header.key, ungrouped, isLastBlock = true, favoriteGatewayIds)
}

private fun MutableList<ServerRow>.addGateways(keyPrefix: String, gateways: List<NymGateway>, isLastBlock: Boolean, favoriteGatewayIds: Set<String>) {
	gateways.forEachIndexed { index, gateway ->
		add(
			ServerRow.Gateway(
				key = "${keyPrefix}_${gateway.identity}",
				gateway = gateway,
				isFavorite = gateway.identity in favoriteGatewayIds,
				isLastInGroup = isLastBlock && index == gateways.lastIndex,
			),
		)
	}
}

private fun ItemType.CountryItem.isExpandedByDefault(query: String, selectedKey: String?): Boolean {
	val countryCode = locale.country.lowercase()
	return gateways.any {
		it.identity == selectedKey ||
			it.region.equals(selectedKey, true) ||
			(
				countryCode == "us" &&
					query.takeIf { q -> q.isNotBlank() }?.let { q ->
						it.region?.contains(q, true)
					} ?: false
				)
	}
}

private fun List<NymGateway>.uniqueRows(): List<NymGateway> = distinctBy { it.identity }
