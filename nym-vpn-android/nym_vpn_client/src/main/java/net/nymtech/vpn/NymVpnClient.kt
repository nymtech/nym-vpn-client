package net.nymtech.vpn

import android.content.Context
import android.content.Intent
import android.net.VpnService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.logcathelper.LogcatHelper
import net.nymtech.logcathelper.model.LogLevel
import net.nymtech.vpn.model.VpnClientState
import net.nymtech.vpn.model.Environment
import net.nymtech.vpn.model.ErrorState
import net.nymtech.vpn.model.VpnMode
import net.nymtech.vpn.model.VpnState
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn.util.InvalidCredentialException
import net.nymtech.vpn.util.ServiceManager
import net.nymtech.vpn.util.safeCollect
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import nym_vpn_lib.FfiException
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
	internal object NymVpn : VpnClient {

		private val ioDispatcher = Dispatchers.IO

		override var entryPoint: EntryPoint = NymVpnClientInit.entryPoint
		override var exitPoint: ExitPoint = NymVpnClientInit.exitPoint
		override var mode: VpnMode = NymVpnClientInit.mode
		private val environment: Environment = NymVpnClientInit.environment

		private var logsJob: Job? = null
		private var statsJob: Job? = null

		private val _state = MutableStateFlow(VpnClientState())
		override val stateFlow: Flow<VpnClientState> = _state.asStateFlow()

		override suspend fun validateCredential(credential: String): Result<Instant?> {
			return withContext(ioDispatcher) {
				try {
					val expiry = checkCredential(credential)
					Result.success(expiry)
				} catch (_: FfiException) {
					Result.failure(InvalidCredentialException("Credential invalid or expired"))
				}
			}
		}

		override suspend fun start(context: Context, credential: String, foreground: Boolean): Result<Unit> {
			return withContext(ioDispatcher) {
				validateCredential(credential).onFailure {
					return@withContext Result.failure(it)
				}
				if (_state.value.vpnState == VpnState.Down) {
					setVpnState(VpnState.Connecting.InitializingClient)
					clearErrorStatus()
					if (foreground) ServiceManager.startVpnServiceForeground(context) else ServiceManager.startVpnService(context)
				}
				Result.success(Unit)
			}
		}

		override suspend fun stop(context: Context, foreground: Boolean) {
			withContext(ioDispatcher) {
				clearStatisticState()
				setVpnState(VpnState.Disconnecting)
				try {
					stopVpn()
				} catch (e: FfiException) {
					Timber.e(e)
				}
				delay(1000)
				handleClientShutdown(context)
			}
		}

		private fun handleClientShutdown(context: Context) {
			ServiceManager.stopVpnService(context)
			clearStatisticState()
			cancelJobs()
		}

		override fun prepare(context: Context): Intent? {
			return VpnService.prepare(context)
		}
		override fun getState(): VpnClientState {
			return _state.value
		}

		private fun cancelJobs() {
			statsJob?.cancel()
			logsJob?.cancel()
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

		internal fun setVpnState(state: VpnState) {
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

		internal suspend fun connect(context: Context) {
			cancelJobs()
			withContext(ioDispatcher) {
				logsJob = launch(ioDispatcher) {
					monitorLogs(context)
				}

				statsJob = launch {
					startConnectionTimer()
				}

				try {
					runVpn(
						VpnConfig(
							environment.apiUrl,
							environment.explorerUrl,
							entryPoint,
							exitPoint,
							isTwoHop(mode),
							null,
						),
					)
				} catch (e: FfiException) {
					Timber.e(e)
					setErrorState(ErrorState.GatewayLookupFailure)
					handleClientShutdown(context)
				}
			}
		}

		private suspend fun monitorLogs(context: Context) {
			LogcatHelper.init(context = context).liveLogs.safeCollect {
				if (it.tag.contains(Constants.NYM_VPN_LIB_TAG)) {
					when (it.level) {
						LogLevel.ERROR -> {
							parseErrorMessageForState(it.message) { handleClientShutdown(context) }
						}
						LogLevel.INFO -> {
							parseInfoMessageForState(it.message)
						}
						else -> Unit
					}
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

		private fun parseInfoMessageForState(message: String) {
			// TODO make this more robust in the future
			with(message) {
				when {
					contains("Mixnet processor is running") -> setVpnState(VpnState.Up)
					contains("Setting up connection monitor") -> setVpnState(VpnState.Up)
					contains(
						"Obtaining initial network topology",
					) -> setVpnState(VpnState.Connecting.EstablishingConnection)
				}
			}
		}

		private fun parseErrorMessageForState(message: String, onError: () -> Unit) {
			with(message) {
				val errorState = when {
					contains("failed to lookup described gateways") -> ErrorState.GatewayLookupFailure
					contains("invalid peer certificate") -> ErrorState.BadGatewayPeerCertificate
					contains("No address associated with hostname") -> ErrorState.BadGatewayNoHostnameAddress
					contains("halted unexpectedly") -> ErrorState.VpnHaltedUnexpectedly(message)
					else -> null
				}
				errorState?.let {
					setErrorState(it)
					onError()
				}
			}
		}
	}
}
