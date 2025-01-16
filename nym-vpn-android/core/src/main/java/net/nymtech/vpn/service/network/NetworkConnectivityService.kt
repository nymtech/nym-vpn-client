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

			var wifiState: Int = 0
			var ethernetState: Int = 0
			var cellularState: Int = 0

			override fun onAvailable(network: Network) {
				updateCapabilityState(1, network)
				trySend(NetworkStatus.Connected)
			}

			override fun onLost(network: Network) {
				updateCapabilityState(0, network)
				if (wifiState == 0 && ethernetState == 0 && cellularState == 0) {
					trySend(NetworkStatus.Disconnected)
				}
			}

			override fun onCapabilitiesChanged(network: Network, networkCapabilities: NetworkCapabilities) {
				updateCapabilityState(1, network)
				trySend(NetworkStatus.Connected)
			}
			fun updateCapabilityState(state: Int, network: Network) {
				with(connectivityManager.getNetworkCapabilities(network)) {
					when {
						this == null -> return
						hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> wifiState = state
						hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> cellularState = state
						hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> ethernetState = state
					}
				}
			}
		}

		val request = NetworkRequest.Builder()
			.addCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
			.addTransportType(NetworkCapabilities.TRANSPORT_WIFI)
			.addTransportType(NetworkCapabilities.TRANSPORT_ETHERNET)
			.addTransportType(NetworkCapabilities.TRANSPORT_CELLULAR)
			.build()

		connectivityManager.registerNetworkCallback(request, connectivityCallback)

		awaitClose {
			connectivityManager.unregisterNetworkCallback(connectivityCallback)
		}
	}.distinctUntilChanged().flowOn(Dispatchers.IO)
}
