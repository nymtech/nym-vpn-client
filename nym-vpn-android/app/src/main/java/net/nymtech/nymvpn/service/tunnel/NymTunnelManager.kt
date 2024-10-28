package net.nymtech.nymvpn.service.tunnel

import android.content.Context
import android.net.VpnService
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.qualifiers.ApplicationScope
import net.nymtech.nymvpn.service.notification.NotificationService
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.extensions.toUserMessage
import net.nymtech.vpn.backend.Backend
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import nym_vpn_lib.BandwidthEvent
import nym_vpn_lib.BandwidthStatus.NoBandwidth
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Provider

class NymTunnelManager @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val notificationService: NotificationService,
	private val backend: Provider<Backend>,
	private val context: Context,
	@ApplicationScope private val applicationScope: CoroutineScope,
) : TunnelManager {

	private val _state = MutableStateFlow(TunnelState())
	override val stateFlow: Flow<TunnelState> = _state.onStart {
		_state.update {
			it.copy(isMnemonicStored = isMnemonicStored())
		}
	}.stateIn(applicationScope, SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT), TunnelState())

	@get:Synchronized @set:Synchronized
	private var running: Boolean = false

	override fun getState(): Tunnel.State {
		return backend.get().getState()
	}

	override suspend fun stop() {
		runCatching {
			backend.get().stop()
			running = false
		}
	}

	override suspend fun start(fromBackground: Boolean) {
		runCatching {
			if (running) return Timber.w("Vpn already running")
			if (!isMnemonicStored()) return onMissingMnemonic()
			val intent = VpnService.prepare(context)
			if (intent != null) return launchVpnPermissionNotification()
			val entryCountry = settingsRepository.getFirstHopCountry()
			val exitCountry = settingsRepository.getLastHopCountry()
			val tunnel = NymTunnel(
				entryPoint = entryCountry.toEntryPoint(),
				exitPoint = exitCountry.toExitPoint(),
				mode = settingsRepository.getVpnMode(),
				environment = settingsRepository.getEnvironment(),
				statChange = ::emitStats,
				stateChange = ::onStateChange,
				backendMessage = ::onBackendMessage,
			)
			backend.get().start(tunnel, fromBackground)
			running = true
		}
	}

	override suspend fun storeMnemonic(mnemonic: String) {
		backend.get().storeMnemonic(mnemonic)
		emitMnemonicStored(true)
	}

	override suspend fun isMnemonicStored(): Boolean {
		return backend.get().isMnemonicStored()
	}

	override suspend fun removeMnemonic() {
		backend.get().removeMnemonic()
		emitMnemonicStored(false)
	}

	private fun emitMnemonicStored(stored: Boolean) {
		_state.update {
			it.copy(isMnemonicStored = stored)
		}
	}

	private fun onBackendMessage(backendMessage: BackendMessage) {
		launchBackendNotification(backendMessage)
		emitMessage(backendMessage)
	}

	private fun emitMessage(backendMessage: BackendMessage) {
		_state.update {
			it.copy(
				backendMessage = backendMessage,
			)
		}
	}

	private fun emitStats(statistics: Statistics) {
		_state.update {
			it.copy(
				statistics = statistics,
			)
		}
	}

	private fun onStateChange(state: Tunnel.State) {
		if (state == Tunnel.State.Down) running = false
		context.requestTileServiceStateUpdate()
		emitState(state)
	}

	private fun onMissingMnemonic() {
		val message = context.getString(R.string.missing_mnemonic)
		if (NymVpn.isForeground()) {
			// TODO add message for mnemonic
			// emitMessage(BackendMessage.Failure(VpnException.InvalidCredential(details = message)))
		} else {
			launchMnemonicNotification(message)
		}
	}

	private fun emitState(state: Tunnel.State) {
		_state.update {
			it.copy(
				state = state,
			)
		}
	}

	private fun launchVpnPermissionNotification() {
		if (!NymVpn.isForeground()) {
			notificationService.showNotification(
				title = context.getString(R.string.permission_required),
				description = context.getString(R.string.vpn_permission_missing),
			)
		}
	}

	private fun launchMnemonicNotification(description: String) {
		notificationService.showNotification(
			title = context.getString(R.string.connection_failed),
			description = description,
		)
	}

	private fun launchBackendNotification(backendMessage: BackendMessage) {
		when (backendMessage) {
			is BackendMessage.Failure -> {
				// TODO if credential error we might need to handle differently if app is in foreground
				notificationService.showNotification(
					title = context.getString(R.string.connection_failed),
					description = backendMessage.reason.toUserMessage(context),
				)
			}
			is BackendMessage.BandwidthAlert -> {
				when (val alert = backendMessage.status) {
					BandwidthEvent.NoBandwidth -> notificationService.showNotification(
						title = context.getString(R.string.bandwidth_alert),
						description = context.getString(R.string.no_bandwidth),
					)
					is BandwidthEvent.RemainingBandwidth -> notificationService.showNotification(
						title = context.getString(R.string.bandwidth_alert),
						description = context.getString(R.string.low_bandwidth) + " ${alert.v1}",
					)
				}
			}
			BackendMessage.None -> Unit
		}
	}
}
