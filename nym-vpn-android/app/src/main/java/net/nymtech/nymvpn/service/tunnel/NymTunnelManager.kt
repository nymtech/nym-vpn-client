package net.nymtech.nymvpn.service.tunnel

import android.content.Context
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.plus
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.qualifiers.ApplicationScope
import net.nymtech.nymvpn.module.qualifiers.IoDispatcher
import net.nymtech.nymvpn.service.notification.NotificationService
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.extensions.toMB
import net.nymtech.nymvpn.util.extensions.toUserMessage
import net.nymtech.vpn.backend.Backend
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import net.nymtech.vpn.util.exceptions.NymVpnInitializeException
import nym_vpn_lib.AccountLinks
import nym_vpn_lib.AccountStateSummary
import nym_vpn_lib.BandwidthEvent
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Provider

class NymTunnelManager @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val notificationService: NotificationService,
	private val backend: Provider<Backend>,
	private val context: Context,
	@ApplicationScope private val applicationScope: CoroutineScope,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : TunnelManager {

	private val _state = MutableStateFlow(TunnelManagerState())
	override val stateFlow: Flow<TunnelManagerState> = _state.onStart {
		val isMnemonicStored = isMnemonicStored()
		_state.update {
			it.copy(
				isMnemonicStored = isMnemonicStored,
				accountLinks = if (isMnemonicStored) getAccountLinks() else null,
			)
		}
	}.stateIn(applicationScope.plus(ioDispatcher), SharingStarted.Eagerly, TunnelManagerState())

	override fun getState(): Tunnel.State {
		return backend.get().getState()
	}

	override suspend fun stop() {
		runCatching {
			backend.get().stop()
		}
	}

	override suspend fun start(fromBackground: Boolean) {
		runCatching {
			val tunnel = NymTunnel(
				entryPoint = getEntryPoint(),
				exitPoint = getExitPoint(),
				mode = settingsRepository.getVpnMode(),
				environment = settingsRepository.getEnvironment(),
				statChange = ::emitStats,
				stateChange = ::onStateChange,
				backendMessage = ::onBackendMessage,
				credentialMode = settingsRepository.isCredentialMode(),
			)
			backend.get().start(tunnel, fromBackground)
		}.onFailure {
			if (it is NymVpnInitializeException) {
				when (it) {
					is NymVpnInitializeException.VpnAlreadyRunning -> Timber.w("Vpn already running")
					is NymVpnInitializeException.VpnPermissionDenied -> launchVpnPermissionNotification()
				}
			} else {
				Timber.e(it)
			}
		}
	}

	private suspend fun getEntryPoint(): EntryPoint {
		val isManualGatewaysEnabled = settingsRepository.isManualGatewayOverride()
		val entryCountry = settingsRepository.getFirstHopCountry()
		if (!isManualGatewaysEnabled) return entryCountry.toEntryPoint()
		val gatewayId = settingsRepository.getEntryGatewayId() ?: return entryCountry.toEntryPoint()
		return try {
			EntryPoint.Gateway(identity = gatewayId)
		} catch (e: Exception) {
			Timber.e(e)
			entryCountry.toEntryPoint()
		}
	}

	private suspend fun getExitPoint(): ExitPoint {
		val isManualGatewaysEnabled = settingsRepository.isManualGatewayOverride()
		val exitCountry = settingsRepository.getLastHopCountry()
		if (!isManualGatewaysEnabled) return exitCountry.toExitPoint()
		val gatewayId = settingsRepository.getExitGatewayId() ?: return exitCountry.toExitPoint()
		return try {
			ExitPoint.Gateway(identity = gatewayId)
		} catch (e: Exception) {
			Timber.e(e)
			exitCountry.toExitPoint()
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

	override suspend fun getAccountSummary(): AccountStateSummary {
		return backend.get().getAccountSummary()
	}

	override suspend fun getAccountLinks(): AccountLinks? {
		return try {
			backend.get().getAccountLinks(settingsRepository.getEnvironment())
		} catch (_: Exception) {
			null
		}
	}

	private fun emitMnemonicStored(stored: Boolean) {
		_state.update {
			it.copy(isMnemonicStored = stored)
		}
	}

	private fun onBackendMessage(backendMessage: BackendMessage) {
		launchBackendNotification(backendMessage)
		emitMessage(backendMessage)
		// TODO For now, we'll stop tunnel on errors
		if (backendMessage is BackendMessage.Failure) {
			Timber.d("Shutting tunnel down on fatal error")
			applicationScope.launch(ioDispatcher) {
				backend.get().stop()
			}
		}
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
				tunnelStatistics = statistics,
			)
		}
	}

	private fun onStateChange(state: Tunnel.State) {
		Timber.d("Requesting tile update with new state: $state")
		context.requestTileServiceStateUpdate()
		emitState(state)
	}

	private fun emitState(state: Tunnel.State) {
		_state.update {
			it.copy(
				tunnelState = state,
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

	private fun launchBackendNotification(backendMessage: BackendMessage) {
		when (backendMessage) {
			is BackendMessage.Failure -> {
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
						description = context.getString(R.string.low_bandwidth) + " ${alert.v1.toMB()} MB",
					)
				}
			}
			BackendMessage.None -> Unit
			is BackendMessage.StartFailure -> notificationService.showNotification(
				title = context.getString(R.string.connection_failed),
				description = backendMessage.exception.toUserMessage(context),
			)
		}
	}
}
