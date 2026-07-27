package net.nymtech.nymvpn.manager.favorites

import android.content.Context
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import nym_vpn_lib.FavoritesController
import nym_vpn_lib_types.FavoriteSelector
import nym_vpn_lib_types.FavoriteSelectors
import timber.log.Timber
import javax.inject.Inject

class NymFavoritesManager @Inject constructor(
	@ApplicationContext private val context: Context,
	@ApplicationScope private val applicationScope: CoroutineScope,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : FavoritesManager {

	companion object {
		private const val TAG = "favorites-manager"
	}

	private val _favoritesFlow = MutableStateFlow(FavoriteSelectors(entry = emptyList(), exit = emptyList()))
	override val favoritesFlow: StateFlow<FavoriteSelectors> = _favoritesFlow.asStateFlow()

	private val controllerMutex = Mutex()

	@Volatile
	private var controller: FavoritesController? = null

	init {
		applicationScope.launch(ioDispatcher) { refresh() }
	}

	private suspend fun getController(): FavoritesController {
		controller?.let { return it }
		return controllerMutex.withLock {
			controller ?: FavoritesController.open(context.filesDir.absolutePath).also { controller = it }
		}
	}

	override suspend fun addFavoriteEntry(selector: FavoriteSelector) = mutate { it.addFavoriteEntry(selector) }

	override suspend fun addFavoriteExit(selector: FavoriteSelector) = mutate { it.addFavoriteExit(selector) }

	override suspend fun removeFavoriteEntry(selector: FavoriteSelector) = mutate { it.removeFavoriteEntry(selector) }

	override suspend fun removeFavoriteExit(selector: FavoriteSelector) = mutate { it.removeFavoriteExit(selector) }

	private suspend fun refresh() {
		runCatching { getController().getFavorites() }
			.onSuccess { _favoritesFlow.value = it }
			.onFailure { Timber.tag(TAG).e(it, "RefreshFavoritesFailed") }
	}

	private suspend fun mutate(block: suspend (FavoritesController) -> Unit) {
		runCatching { block(getController()) }
			.onFailure { Timber.tag(TAG).e(it, "FavoritesMutationFailed") }
		refresh()
	}
}
