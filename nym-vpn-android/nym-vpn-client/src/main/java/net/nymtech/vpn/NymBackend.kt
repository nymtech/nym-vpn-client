package net.nymtech.vpn

import android.content.Context
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.vpn.model.Statistics
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn.util.InvalidCredentialException
import net.nymtech.vpn.util.ServiceManager
import nym_vpn_lib.TunStatus
import nym_vpn_lib.TunnelStatusListener
import nym_vpn_lib.VpnConfig
import nym_vpn_lib.checkCredential
import nym_vpn_lib.runVpn
import java.time.Instant

object NymBackend : Backend, TunnelStatusListener {

	private val ioDispatcher = Dispatchers.IO

	private var statsJob: Job? = null
	private var tunnel: Tunnel? = null
	private var state: Tunnel.State = Tunnel.State.Down

	override suspend fun validateCredential(credential: String): Instant? {
		return try {
			withContext(ioDispatcher) {
				checkCredential(credential)
			}
		} catch (e: Exception) {
			throw InvalidCredentialException("Credential invalid or expired")
		}
	}

	override suspend fun importCredential(credential: String): Instant? {
		return try {
			nym_vpn_lib.importCredential(credential, Constants.NATIVE_STORAGE_PATH)
		} catch (e: Exception) {
			throw InvalidCredentialException("Credential invalid or expired")
		}
	}

	override suspend fun start(context: Context, tunnel: Tunnel): Tunnel.State {
		this.tunnel = tunnel
		// set all env vars
		tunnel.environment.setup()
		withContext(ioDispatcher) {
			ServiceManager.startVpnService(context)
		}
		return Tunnel.State.Connecting.InitializingClient
	}

	override fun stop(context: Context): Tunnel.State {
		ServiceManager.stopVpnService(context)
		return Tunnel.State.Disconnecting
	}

	private fun onDisconnect() {
		statsJob?.cancel()
		tunnel?.onStatisticChange(Statistics())
	}

	private fun onConnect() = CoroutineScope(ioDispatcher).launch {
		startConnectionTimer()
	}

	override fun getState(): Tunnel.State {
		return state
	}

	private fun isTwoHop(mode: Tunnel.Mode): Boolean = when (mode) {
		Tunnel.Mode.TWO_HOP_MIXNET -> true
		else -> false
	}

	internal suspend fun connect() {
		withContext(ioDispatcher) {
			tunnel?.let {
				runVpn(
					VpnConfig(
						it.environment.apiUrl,
						it.environment.explorerUrl,
						it.entryPoint,
						it.exitPoint,
						isTwoHop(it.mode),
						Constants.NATIVE_STORAGE_PATH,
						this@NymBackend,
					),
				)
			}
		}
	}

	private suspend fun startConnectionTimer() {
		withContext(ioDispatcher) {
			var seconds = 0L
			do {
				if (state == Tunnel.State.Up) {
					tunnel?.onStatisticChange(Statistics(seconds))
					seconds++
				}
				delay(Constants.STATISTICS_INTERVAL_MILLI)
			} while (true)
		}
	}

	override fun onTunStatusChange(status: TunStatus) {
		val state = when (status) {
			TunStatus.INITIALIZING_CLIENT -> Tunnel.State.Connecting.InitializingClient
			TunStatus.ESTABLISHING_CONNECTION -> Tunnel.State.Connecting.EstablishingConnection
			TunStatus.DOWN -> {
				Tunnel.State.Down
			}
			TunStatus.UP -> {
				statsJob = onConnect()
				Tunnel.State.Up
			}
			TunStatus.DISCONNECTING -> {
				onDisconnect()
				Tunnel.State.Disconnecting
			}
		}
		this.state = state
		tunnel?.onStateChange(state)
	}
}
