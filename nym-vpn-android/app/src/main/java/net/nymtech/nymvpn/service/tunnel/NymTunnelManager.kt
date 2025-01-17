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
import net.nymtech.nymvpn.service.tunnel.model.BackendUiEvent
import net.nymtech.nymvpn.service.tunnel.model.MixnetConnectionState
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.extensions.toMB
import net.nymtech.nymvpn.util.extensions.toUserAgent
import net.nymtech.nymvpn.util.extensions.toUserMessage
import net.nymtech.vpn.backend.Backend
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendEvent
import net.nymtech.vpn.util.exceptions.BackendException
import nym_vpn_lib.AccountLinks
import nym_vpn_lib.AccountStateSummary
import nym_vpn_lib.BandwidthEvent
import nym_vpn_lib.ConnectionData
import nym_vpn_lib.ConnectionEvent
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ErrorStateReason
import nym_vpn_lib.ExitPoint
import nym_vpn_lib.MixnetEvent
import nym_vpn_lib.TunnelState
import nym_vpn_lib.VpnException
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
		val deviceId = if (isMnemonicStored) getDeviceId() else null
		_state.update {
			it.copy(
				isMnemonicStored = isMnemonicStored,
				deviceId = deviceId,
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

	override suspend fun start() {
		runCatching {
			// clear any error states
			emitBackendUiEvent(null)
			val tunnel = NymTunnel(
				entryPoint = getEntryPoint(),
				exitPoint = getExitPoint(),
				mode = settingsRepository.getVpnMode(),
				environment = settingsRepository.getEnvironment(),
				stateChange = ::onStateChange,
				backendEvent = ::onBackendEvent,
				credentialMode = settingsRepository.isCredentialMode(),
			)
			backend.get().start(tunnel, context.toUserAgent())
		}.onFailure {
			if (it is BackendException) {
				when (it) {
					is BackendException.VpnAlreadyRunning -> Timber.w("Vpn already running")
					is BackendException.VpnPermissionDenied -> launchVpnPermissionNotification()
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
		updateDeviceId()
		refreshAccountLinks()
	}

	override suspend fun isMnemonicStored(): Boolean {
		return backend.get().isMnemonicStored()
	}

	override suspend fun removeMnemonic() {
		backend.get().removeMnemonic()
		emitMnemonicStored(false)
		refreshAccountLinks()
	}

	private suspend fun updateDeviceId() {
		runCatching {
			val id = getDeviceId()
			_state.update {
				it.copy(deviceId = id)
			}
		}.onFailure {
			Timber.e(it)
		}
	}

	private suspend fun getDeviceId(): String {
		return backend.get().getDeviceIdentity()
	}

	override suspend fun getAccountSummary(): AccountStateSummary {
		return backend.get().getAccountSummary()
	}

	override suspend fun getAccountLinks(): AccountLinks? {
		return try {
			backend.get().getAccountLinks()
		} catch (_: Exception) {
			null
		}
	}

	override suspend fun refreshAccountLinks() {
		val accountLinks = getAccountLinks()
		_state.update {
			it.copy(accountLinks = accountLinks)
		}
	}

	private fun emitMnemonicStored(stored: Boolean) {
		_state.update {
			it.copy(isMnemonicStored = stored)
		}
	}

	private fun emitBackendUiEvent(backendEvent: BackendUiEvent?) {
		_state.update {
			it.copy(backendUiEvent = backendEvent)
		}
	}

	private fun emitConnectionData(connectionData: ConnectionData?) {
		_state.update {
			it.copy(connectionData = connectionData)
		}
	}

	private fun emitMixnetConnectionEvent(connectionEvent: ConnectionEvent) {
		_state.update {
			it.copy(mixnetConnectionState = it.mixnetConnectionState?.onEvent(connectionEvent) ?: MixnetConnectionState().onEvent(connectionEvent))
		}
	}

	private fun onBackendEvent(backendEvent: BackendEvent) {
		when (backendEvent) {
			is BackendEvent.Mixnet -> when (val event = backendEvent.event) {
				is MixnetEvent.Bandwidth -> {
					emitBackendUiEvent(BackendUiEvent.BandwidthAlert(event.v1))
					launchBandwidthNotification(event.v1)
				}
				is MixnetEvent.Connection -> emitMixnetConnectionEvent(event.v1)
				is MixnetEvent.ConnectionStatistics -> Timber.d("Stats: ${event.v1}")
			}

			is BackendEvent.StartFailure -> {
				emitBackendUiEvent(BackendUiEvent.StartFailure(backendEvent.exception))
				launchStartFailureNotification(backendEvent.exception)
			}
			is BackendEvent.Tunnel -> when (val state = backendEvent.state) {
				is TunnelState.Connected -> emitConnectionData(state.connectionData)
				is TunnelState.Connecting -> emitConnectionData(state.connectionData)
				is TunnelState.Disconnecting -> Timber.d("After disconnect status: ${state.afterDisconnect.name}")
				is TunnelState.Error -> {
					Timber.d("Shutting tunnel down on fatal error")
					emitBackendUiEvent(BackendUiEvent.Failure(state.v1))
					launchBackendFailureNotification(state.v1)
					applicationScope.launch(ioDispatcher) {
						backend.get().stop()
					}
				}
				else -> Unit
			}
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

	private fun launchBandwidthNotification(bandwidthEvent: BandwidthEvent) {
		when (bandwidthEvent) {
			BandwidthEvent.NoBandwidth -> notificationService.showNotification(
				title = context.getString(R.string.bandwidth_alert),
				description = context.getString(R.string.no_bandwidth),
			)
			is BandwidthEvent.RemainingBandwidth -> notificationService.showNotification(
				title = context.getString(R.string.bandwidth_alert),
				description = context.getString(R.string.low_bandwidth) + " ${bandwidthEvent.v1.toMB()} MB",
			)
		}
	}

	private fun launchStartFailureNotification(exception: VpnException) {
		notificationService.showNotification(
			title = context.getString(R.string.connection_failed),
			description = exception.toUserMessage(context),
		)
	}

	private fun launchBackendFailureNotification(reason: ErrorStateReason) {
		notificationService.showNotification(
			title = context.getString(R.string.connection_failed),
			description = reason.toUserMessage(context),
		)
	}
}
