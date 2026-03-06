package net.nymtech.vpn.backend

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.connectivity.NetworkConnectivityService
import net.nymtech.connectivity.NetworkStatus
import net.nymtech.vpn.backend.service.VpnService
import nym_vpn_lib.ConnectivityObserver

/**
 * Bridges OS connectivity to Rust observers.
 */
internal class VpnConnectivityBridge(private val service: VpnService, private val scope: CoroutineScope) {
	private val observers: MutableList<ConnectivityObserver> = mutableListOf()

	@Volatile private var networkStatus: NetworkStatus = NetworkStatus.Unknown

	fun start() {
		scope.launch {
			NetworkConnectivityService(service).networkStatus.collect { status ->
				networkStatus = status
				notifyObservers()
			}
		}
	}

	fun addObserver(observer: ConnectivityObserver) {
		if (!observers.any { it.id() == observer.id() }) {
			observers.add(observer)
			notifyObservers()
		}
	}

	fun removeObserver(observer: ConnectivityObserver) {
		observers.removeIf { it.id() == observer.id() }
	}

	private fun notifyObservers() {
		val isConnected = when (networkStatus) {
			NetworkStatus.Connected -> true
			NetworkStatus.Disconnected -> false
			NetworkStatus.Unknown -> return
		}
		observers.forEach { it.onNetworkChange(isConnected) }
	}
}
