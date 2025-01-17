package net.nymtech.connectivity

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn

class NetworkConnectivityService(context: Context) : NetworkService {

	private val connectivityManager = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

	@OptIn(FlowPreview::class)
	override val networkStatus: Flow<NetworkStatus> = callbackFlow {

		var wifiState: Int = 0
		var ethernetState: Int = 0
		var cellularState: Int = 0

		val currentNetwork = connectivityManager.activeNetwork
		if (currentNetwork == null) {
			// all networks unavailable or airplane mode on
			trySend(NetworkStatus.Disconnected)
		}

		// Listen for Airplane Mode changes
		val airplaneModeReceiver = object : BroadcastReceiver() {
			override fun onReceive(context: Context, intent: Intent) {
				if (intent.action == Intent.ACTION_AIRPLANE_MODE_CHANGED) {
					val isAirplaneModeOn = intent.getBooleanExtra("state", false)
					if (isAirplaneModeOn && wifiState == 0) {
						trySend(NetworkStatus.Disconnected)
					}
				}
			}
		}

		context.registerReceiver(airplaneModeReceiver, IntentFilter(Intent.ACTION_AIRPLANE_MODE_CHANGED))

		val connectivityCallback = object : ConnectivityManager.NetworkCallback() {

			override fun onAvailable(network: Network) {
				updateCapabilityState(1, network)
				trySend(NetworkStatus.Connected)
			}

			override fun onUnavailable() {
				val currentNetwork = connectivityManager.activeNetwork
				if (currentNetwork == null) {
					// all networks unavailable or airplane mode on
					trySend(NetworkStatus.Disconnected)
				}
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
						hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) ->
							cellularState =
								state

						hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) ->
							ethernetState =
								state
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
			context.unregisterReceiver(airplaneModeReceiver)
		}
	}.distinctUntilChanged().flowOn(Dispatchers.IO).debounce(1000L)
}
