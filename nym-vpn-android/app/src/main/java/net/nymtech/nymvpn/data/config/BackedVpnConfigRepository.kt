package net.nymtech.nymvpn.data.config

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.manager.backend.VpnServiceConnectionManager
import net.nymtech.vpn.model.config.CoreVpnConfig
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import net.nymtech.vpn.model.config.ConfigResult
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class BackedVpnConfigRepository @Inject constructor(
	private val serviceConnectionManager: VpnServiceConnectionManager,
	@ApplicationScope private val appScope: CoroutineScope,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : VpnConfigRepository {

	companion object Companion {
		private const val TAG = "svc-vpn-config-repo"
	}

	private val _config = MutableStateFlow(CoreVpnConfig())
	override val configFlow: Flow<CoreVpnConfig> = _config.asStateFlow()

	init {
		appScope.launch(ioDispatcher) {
			runCatching {
				_config.value = serviceConnectionManager.withApi { it.getConfig() }
			}.onFailure { Timber.tag(TAG).e(it, "Initial getConfig failed") }
		}
	}

	override suspend fun getConfig(): CoreVpnConfig {
		val cfg = serviceConnectionManager.withApi { it.getConfig() }
		_config.value = cfg
		return cfg
	}

	override suspend fun apply(updates: List<CoreVpnConfigUpdate>): ConfigResult {
		val res = serviceConnectionManager.withApi { it.applyUpdates(updates) }
		when (res) {
			is ConfigResult.Ok -> _config.value = res.updated
			is ConfigResult.Failed ->
				Timber.tag(TAG).e("apply failed: %s (%s)", res.message, res.cause)
		}
		return res
	}

	override suspend fun apply(update: CoreVpnConfigUpdate): ConfigResult {
		val res = serviceConnectionManager.withApi { it.applyUpdate(update) }
		when (res) {
			is ConfigResult.Ok -> _config.value = res.updated
			is ConfigResult.Failed ->
				Timber.tag(TAG).e("apply failed: %s (%s)", res.message, res.cause)
		}
		return res
	}
}
