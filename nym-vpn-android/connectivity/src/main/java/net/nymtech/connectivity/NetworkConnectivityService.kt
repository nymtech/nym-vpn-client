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
import timber.log.Timber

class NetworkConnectivityService(context: Context) : NetworkService {
	private val appContext = context.applicationContext
	private val connectivityManager = appContext.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

	@OptIn(FlowPreview::class)
	override val networkStatus: Flow<NetworkStatus> = callbackFlow {

		fun hasInternet(network: Network?): Boolean {
			val capabilities = connectivityManager.getNetworkCapabilities(network)
			return capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) == true &&
				capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)
		}

		val initialNetwork = connectivityManager.activeNetwork
		trySend(if (hasInternet(initialNetwork)) NetworkStatus.Connected else NetworkStatus.Disconnected)

		val airplaneModeReceiver = object : BroadcastReceiver() {
			override fun onReceive(context: Context, intent: Intent) {
				if (intent.action == Intent.ACTION_AIRPLANE_MODE_CHANGED) {
					val isAirplaneModeOn = intent.getBooleanExtra("state", false)
					if (isAirplaneModeOn) {
						val currentNetwork = connectivityManager.activeNetwork
						Timber.d("Airplane Mode: on=true, activeNetwork=$currentNetwork")
						if (currentNetwork == null || !hasInternet(currentNetwork)) {
							Timber.d("Emitting Disconnected due to airplane mode")
							trySend(NetworkStatus.Disconnected)
						}
					}
				}
			}
		}
		appContext.registerReceiver(airplaneModeReceiver, IntentFilter(Intent.ACTION_AIRPLANE_MODE_CHANGED))

		val connectivityCallback = object : ConnectivityManager.NetworkCallback() {
			private val activeNetworksWithInternet = mutableSetOf<Network>()

			override fun onAvailable(network: Network) {
				Timber.d("onAvailable: network=$network")
				if (hasInternet(network)) {
					activeNetworksWithInternet.add(network)
					trySend(NetworkStatus.Connected)
				}
			}

			override fun onUnavailable() {
				Timber.d("onUnavailable")
				activeNetworksWithInternet.clear()
				val currentNetwork = connectivityManager.activeNetwork
				if (currentNetwork == null || !hasInternet(currentNetwork)) {
					trySend(NetworkStatus.Disconnected)
				}
			}

			override fun onLost(network: Network) {
				Timber.d("onLost: network=$network")
				activeNetworksWithInternet.remove(network)
				val currentNetwork = connectivityManager.activeNetwork
				if (currentNetwork == null || !hasInternet(currentNetwork)) {
					Timber.d("Emitting Disconnected due to onLost")
					trySend(NetworkStatus.Disconnected)
				} else if (activeNetworksWithInternet.isEmpty()) {
					trySend(NetworkStatus.Disconnected)
				}
			}

			override fun onCapabilitiesChanged(network: Network, networkCapabilities: NetworkCapabilities) {
				if (networkCapabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) &&
					networkCapabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED)
				) {
					activeNetworksWithInternet.add(network)
					trySend(NetworkStatus.Connected)
				} else {
					activeNetworksWithInternet.remove(network)
					val currentNetwork = connectivityManager.activeNetwork
					if (currentNetwork == null || !hasInternet(currentNetwork)) {
						Timber.d("Emitting Disconnected due to capabilities change")
						trySend(NetworkStatus.Disconnected)
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
			appContext.unregisterReceiver(airplaneModeReceiver)
		}
	}.distinctUntilChanged().flowOn(Dispatchers.IO).debounce(500L)
}
