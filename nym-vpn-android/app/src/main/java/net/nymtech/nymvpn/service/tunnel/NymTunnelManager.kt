package net.nymtech.nymvpn.service.tunnel

import android.content.Context
import android.net.VpnService
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.util.extensions.isInvalid
import net.nymtech.vpn.backend.Backend
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import nym_vpn_lib.VpnException
import timber.log.Timber
import java.time.Instant
import javax.inject.Inject
import javax.inject.Provider

class NymTunnelManager @Inject constructor(
	private val settingsRepository: SettingsRepository,
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

	override suspend fun start() {
		runCatching {
			Timber.d("Starting")
			if (running) return Timber.w("Vpn already running")
			val intent = VpnService.prepare(context)
			if (intent != null) return // TODO handle missing permission
			val credentialExpiry = settingsRepository.getCredentialExpiry()
			if (credentialExpiry.isInvalid()) return emitMessage(BackendMessage.Failure(VpnException.InvalidCredential(details = "Invalid credential")))
			val entryCountry = settingsRepository.getFirstHopCountry()
			val exitCountry = settingsRepository.getLastHopCountry()
			val tunnel = NymTunnel(
				entryPoint = entryCountry.toEntryPoint(),
				exitPoint = exitCountry.toExitPoint(),
				mode = settingsRepository.getVpnMode(),
				environment = settingsRepository.getEnvironment(),
				statChange = ::emitStats,
				stateChange = ::onStateChange,
				backendMessage = ::emitMessage,
			)
			backend.get().start(tunnel, false)
			running = true
		}
	}

	override suspend fun importCredential(credential: String): Result<Instant?> {
		return kotlin.runCatching {
			backend.get().importCredential(credential)
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
				statistics = statistics,
			)
		}
	}

	private fun onStateChange(state: Tunnel.State) {
		if (state == Tunnel.State.Down) running = false
		emitState(state)
	}

	private fun emitState(state: Tunnel.State) {
		_state.update {
			it.copy(
				state = state,
			)
		}
	}
}
