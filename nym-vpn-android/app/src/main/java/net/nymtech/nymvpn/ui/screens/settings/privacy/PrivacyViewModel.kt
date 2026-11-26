package net.nymtech.nymvpn.ui.screens.settings.privacy

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.ui.screens.settings.privacy.components.LoggingLevel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import net.nymtech.vpn.model.config.ConfigResult
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class PrivacyViewModel @Inject constructor(private val vpnConfigRepository: VpnConfigRepository, private val settingsRepository: SettingsRepository) : ViewModel() {

	private val loggingLevelLock = Mutex()

	fun onNetworkStatsEnabled(enabled: Boolean) = viewModelScope.launch {
		settingsRepository.setStatisticsEnabled(enabled)
		vpnConfigRepository.apply(CoreVpnConfigUpdate.SetNetworkStatsEnabled(enabled))
	}

	fun onMonitoringEnabled(enabled: Boolean) = viewModelScope.launch {
		vpnConfigRepository.apply(CoreVpnConfigUpdate.SetSentry(enabled))
	}

	/**
	 * The level lives in two stores with no transaction between them, so the writes go in the order
	 * that leaves a state the card shows honestly if the second one never happens: the debug flag goes
	 * on last and off first. A failed debug write stops the change. Selections apply one at a time.
	 */
	fun onLoggingLevelChange(level: LoggingLevel) = viewModelScope.launch {
		loggingLevelLock.withLock {
			runCatching {
				if (level == LoggingLevel.DEBUG) {
					settingsRepository.setLogsEnabled(true)
					applyDebugLog(true)
				} else if (applyDebugLog(false)) {
					settingsRepository.setLogsEnabled(level != LoggingLevel.OFF)
				}
			}.onFailure {
				Timber.e(it, "Failed to update logging level")
			}
		}
	}

	/** The core picks up a debug change only when the process restarts, in either direction. */
	fun isRestartPending(level: LoggingLevel): Boolean {
		val debugAtStart = NymVpn.coreDebugLogAtStart ?: return false
		return debugAtStart != (level == LoggingLevel.DEBUG)
	}

	private suspend fun applyDebugLog(enabled: Boolean): Boolean {
		val result = vpnConfigRepository.apply(CoreVpnConfigUpdate.SetDebugLog(enabled))
		if (result is ConfigResult.Failed) Timber.e("Failed to update debug logging: ${result.message}")
		return result is ConfigResult.Ok
	}
}
