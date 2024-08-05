package net.nymtech.nymvpn.service.tunnel

import android.content.Context
import android.net.VpnService
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.util.extensions.isExpired
import net.nymtech.vpn.Backend
import net.nymtech.vpn.Tunnel
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import net.nymtech.vpn.util.InvalidCredentialException
import net.nymtech.vpn.util.MissingPermissionException
import javax.inject.Inject
import javax.inject.Provider

class NymTunnelManager @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val backend: Provider<Backend>,
) : TunnelManager {

	private val _state = MutableStateFlow(TunnelState())
	override val stateFlow: Flow<TunnelState> = _state.asStateFlow()

	override suspend fun stopVpn(context: Context): Result<Tunnel.State> {
		return runCatching {
			backend.get().stop(context)
		}
	}

	override suspend fun startVpn(context: Context): Result<Tunnel.State> {
		return runCatching {
			val intent = VpnService.prepare(context)
			if (intent != null) return Result.failure(MissingPermissionException("VPN permission missing"))
			val entryCountry = settingsRepository.getFirstHopCountry()
			val exitCountry = settingsRepository.getLastHopCountry()
			val credentialExpiry = settingsRepository.getCredentialExpiry()
			val mode = settingsRepository.getVpnMode()
			val tunnel = NymTunnel(
				entryPoint = entryCountry.toEntryPoint(),
				exitPoint = exitCountry.toExitPoint(),
				mode = mode,
				environment = Tunnel.Environment.MAINNET,
				statChange = ::emitStats,
				stateChange = ::emitState,
				backendMessage = ::emitMessage,
			)
			if (credentialExpiry != null && credentialExpiry.isExpired()) {
				return Result.failure(InvalidCredentialException("Credential missing or expired"))
			}
			backend.get().start(context, tunnel)
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

	private fun emitState(state: Tunnel.State) {
		_state.update {
			it.copy(
				state = state,
			)
		}
	}
}
