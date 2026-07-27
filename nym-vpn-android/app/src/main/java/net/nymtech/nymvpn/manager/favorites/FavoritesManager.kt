package net.nymtech.nymvpn.manager.favorites

import kotlinx.coroutines.flow.StateFlow
import nym_vpn_lib_types.FavoriteSelector
import nym_vpn_lib_types.FavoriteSelectors

interface FavoritesManager {

	val favoritesFlow: StateFlow<FavoriteSelectors>

	suspend fun addFavoriteEntry(selector: FavoriteSelector)

	suspend fun addFavoriteExit(selector: FavoriteSelector)

	suspend fun removeFavoriteEntry(selector: FavoriteSelector)

	suspend fun removeFavoriteExit(selector: FavoriteSelector)
}
