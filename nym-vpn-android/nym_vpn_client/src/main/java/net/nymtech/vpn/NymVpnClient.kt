package net.nymtech.vpn

import android.content.Context
import android.content.Intent
import android.net.VpnService
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.vpn.model.Environment
import net.nymtech.vpn.model.ErrorState
import net.nymtech.vpn.model.VpnClientState
import net.nymtech.vpn.model.VpnMode
import net.nymtech.vpn.model.VpnState
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn.util.InvalidCredentialException
import net.nymtech.vpn.util.ServiceManager
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import nym_vpn_lib.TunStatus
import nym_vpn_lib.TunnelStatusListener
import nym_vpn_lib.VpnConfig
import nym_vpn_lib.checkCredential
import nym_vpn_lib.runVpn
import nym_vpn_lib.stopVpn
import timber.log.Timber
import java.time.Instant

object NymVpnClient {

	private object NymVpnClientInit {
		lateinit var entryPoint: EntryPoint
		lateinit var exitPoint: ExitPoint
		lateinit var mode: VpnMode
		lateinit var environment: Environment
	}

	fun init(
		entryPoint: EntryPoint = EntryPoint.Location(
			Constants.DEFAULT_COUNTRY_ISO,
		),
		exitPoint: ExitPoint = ExitPoint.Location(
			Constants.DEFAULT_COUNTRY_ISO,
		),
		mode: VpnMode = VpnMode.TWO_HOP_MIXNET,
		environment: Environment = Environment.MAINNET,
	): VpnClient {
		synchronized(NymVpnClientInit) {
			NymVpnClientInit.entryPoint = entryPoint
			NymVpnClientInit.exitPoint = exitPoint
			NymVpnClientInit.mode = mode
			NymVpnClientInit.environment = environment
			when (NymVpnClientInit.environment) {
				Environment.MAINNET -> Constants.setupEnvironmentMainnet()
				Environment.SANDBOX -> Constants.setupEnvironmentSandbox()
			}

			return NymVpn
		}
	}
	internal object NymVpn : VpnClient, TunnelStatusListener {

		private val ioDispatcher = Dispatchers.IO

		override var entryPoint: EntryPoint = NymVpnClientInit.entryPoint
		override var exitPoint: ExitPoint = NymVpnClientInit.exitPoint
		override var mode: VpnMode = NymVpnClientInit.mode
		private val environment: Environment = NymVpnClientInit.environment

		private var statsJob: Job? = null

		private val _state = MutableStateFlow(VpnClientState())
		override val stateFlow: Flow<VpnClientState> = _state.asStateFlow()

		override suspend fun validateCredential(credential: String): Result<Instant?> {
			return withContext(ioDispatcher) {
				runCatching {
					checkCredential(credential)
				}.onFailure {
					return@withContext Result.failure(InvalidCredentialException("Credential invalid or expired"))
				}
			}
		}

		override suspend fun start(context: Context, credential: String, foreground: Boolean): Result<Unit> {
			return withContext(ioDispatcher) {
				validateCredential(credential).onFailure {
					return@withContext Result.failure(it)
				}
				if (_state.value.vpnState == VpnState.Down) {
					clearErrorStatus()
					if (foreground) ServiceManager.startVpnServiceForeground(context) else ServiceManager.startVpnService(context)
				}
				Result.success(Unit)
			}
		}

		override suspend fun stop(foreground: Boolean) {
			withContext(ioDispatcher) {
				runCatching {
					stopVpn()
				}.onFailure {
					Timber.e(it)
				}
			}
		}

		private fun onDisconnect() {
			clearStatisticState()
			statsJob?.cancel()
		}

		private fun onConnect() = CoroutineScope(ioDispatcher).launch {
			startConnectionTimer()
		}

		override fun prepare(context: Context): Intent? {
			return VpnService.prepare(context)
		}
		override fun getState(): VpnClientState {
			return _state.value
		}

		private fun clearErrorStatus() {
			_state.update {
				it.copy(
					errorState = ErrorState.None,
				)
			}
		}

		private fun clearStatisticState() {
			_state.update {
				it.copy(
					statistics =
					_state.value.statistics.copy(
						connectionSeconds = null,
					),
				)
			}
		}

		private fun setStatisticState(seconds: Long) {
			_state.update {
				it.copy(
					statistics =
					_state.value.statistics.copy(
						connectionSeconds = seconds,
					),
				)
			}
		}

		private fun setErrorState(errorState: ErrorState) {
			_state.update {
				it.copy(
					errorState = errorState,
				)
			}
		}

		private fun setVpnState(state: VpnState) {
			if (state != _state.value.vpnState) {
				_state.update {
					it.copy(
						vpnState = state,
					)
				}
			}
		}

		private fun isTwoHop(mode: VpnMode): Boolean = when (mode) {
			VpnMode.TWO_HOP_MIXNET -> true
			else -> false
		}

		internal suspend fun connect() {
			withContext(ioDispatcher) {
				runCatching {
					runVpn(
						VpnConfig(
							environment.apiUrl,
							environment.explorerUrl,
							entryPoint,
							exitPoint,
							isTwoHop(mode),
							null,
							this@NymVpn,
						),
					)
				}.onFailure {
					// TODO better handle error messaging based on failure message
					Timber.e(it)
					setErrorState(ErrorState.GatewayLookupFailure)
				}
			}
		}

		private suspend fun startConnectionTimer() {
			withContext(ioDispatcher) {
				var seconds = 0L
				do {
					if (_state.value.vpnState == VpnState.Up) {
						setStatisticState(seconds)
						seconds++
					}
					delay(1000)
				} while (true)
			}
		}

		override fun onTunStatusChange(status: TunStatus) {
			val vpnState = when (status) {
				TunStatus.INITIALIZING_CLIENT -> VpnState.Connecting.InitializingClient
				TunStatus.ESTABLISHING_CONNECTION -> VpnState.Connecting.EstablishingConnection
				TunStatus.DOWN -> VpnState.Down
				TunStatus.UP -> {
					statsJob = onConnect()
					VpnState.Up
				}
				TunStatus.DISCONNECTING -> {
					onDisconnect()
					VpnState.Disconnecting
				}
			}
			setVpnState(vpnState)
		}
	}
}
