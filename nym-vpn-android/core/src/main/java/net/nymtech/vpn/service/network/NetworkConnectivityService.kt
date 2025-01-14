package net.nymtech.vpn.service.network

import android.content.Context
import android.net.ConnectivityManager
import android.net.ConnectivityManager.NetworkCallback
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn

class NetworkConnectivityService(context: Context) : NetworkService {

	private val connectivityManager = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

	override val networkStatus: Flow<NetworkStatus> = callbackFlow {
		val connectivityCallback = object : NetworkCallback() {
			override fun onAvailable(network: Network) {
				trySend(NetworkStatus.Connected)
			}

			override fun onUnavailable() {
				trySend(NetworkStatus.Disconnected)
			}

			override fun onLost(network: Network) {
				trySend(NetworkStatus.Disconnected)
			}

			override fun onCapabilitiesChanged(network: Network, networkCapabilities: NetworkCapabilities) {
				trySend(NetworkStatus.Connected)
			}
		}

		val request = NetworkRequest.Builder()
			.addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
			.build()

		connectivityManager.registerNetworkCallback(request, connectivityCallback)

		awaitClose {
			connectivityManager.unregisterNetworkCallback(connectivityCallback)
		}
	}.distinctUntilChanged().flowOn(Dispatchers.IO)
}
