package net.nymtech.nymvpn.service.tunnel

import android.content.Context
import android.net.VpnService
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.notification.NotificationService
import net.nymtech.nymvpn.util.extensions.isInvalid
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.extensions.toUserMessage
import net.nymtech.vpn.backend.Backend
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import nym_vpn_lib.BandwidthStatus
import nym_vpn_lib.VpnException
import timber.log.Timber
import java.time.Instant
import javax.inject.Inject
import javax.inject.Provider

class NymTunnelManager @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val notificationService: NotificationService,
	private val backend: Provider<Backend>,
	private val context: Context,
) : TunnelManager {

	private val _state = MutableStateFlow(TunnelState())
	override val stateFlow: Flow<TunnelState> = _state.asStateFlow()

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
			val credentialExpiry = settingsRepository.getCredentialExpiry()
			if (credentialExpiry.isInvalid()) return onInvalidCredential(credentialExpiry)
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

	override suspend fun importCredential(credential: String): Result<Instant?> {
		return kotlin.runCatching {
			backend.get().importCredential(credential)
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

	private fun onInvalidCredential(expiry: Instant?) {
		val message = if (expiry == null) {
			context.getString(R.string.missing_credential)
		} else {
			context.getString(R.string.exception_cred_invalid)
		}
		if (NymVpn.isForeground()) {
			emitMessage(BackendMessage.Failure(VpnException.InvalidCredential(details = message)))
		} else {
			launchCredentialNotification(message)
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

	private fun launchCredentialNotification(description: String) {
		notificationService.showNotification(
			title = context.getString(R.string.credential_failed_message),
			description = description,
		)
	}

	private fun launchBackendNotification(backendMessage: BackendMessage) {
		when (backendMessage) {
			is BackendMessage.Failure -> {
				val launchNotification = when (backendMessage.exception) {
					is VpnException.InvalidCredential -> !NymVpn.isForeground()
					else -> true
				}
				if (launchNotification) {
					notificationService.showNotification(
						title = context.getString(R.string.connection_failed),
						description = backendMessage.exception.toUserMessage(context),
					)
				}
			}
			is BackendMessage.BandwidthAlert -> {
				when (val alert = backendMessage.status) {
					BandwidthStatus.NoBandwidth -> {
						notificationService.showNotification(
							title = context.getString(R.string.bandwidth_alert),
							description = context.getString(R.string.no_bandwidth),
						)
					}

					is BandwidthStatus.RemainingBandwidth -> {
						notificationService.showNotification(
							title = context.getString(R.string.bandwidth_alert),
							description = context.getString(R.string.low_bandwidth) + " ${alert.bandwidth}",
						)
					}
				}
			}
			BackendMessage.None -> Unit
		}
	}
}
