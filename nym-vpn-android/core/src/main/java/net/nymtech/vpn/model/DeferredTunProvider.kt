package net.nymtech.vpn.model

import net.nymtech.vpn.backend.NymBackend
import nym_vpn_lib.AndroidTunProvider
import nym_vpn_lib.ConnectivityObserver
import nym_vpn_lib.TunnelNetworkSettings
import timber.log.Timber

class DeferredTunProvider(private val owner: NymBackend) : AndroidTunProvider {

	@Volatile
	private var delegate: AndroidTunProvider? = null

	fun setDelegate(provider: AndroidTunProvider) {
		delegate = provider
	}

	override fun bypass(socket: Int) {
		delegate?.bypass(socket)
			?: Timber.w("DeferredTunProvider: bypass called before delegate is set")
	}

	override fun configureTunnel(config: TunnelNetworkSettings): Int {
		return delegate?.configureTunnel(config)
			?: run {
				Timber.w("DeferredTunProvider: configureTunnel called before delegate is set")
				-1
			}
	}

	override fun addConnectivityObserver(observer: ConnectivityObserver) {
		owner.addConnectivityObserver(observer)
	}

	override fun removeConnectivityObserver(observer: ConnectivityObserver) {
		owner.removeObserver(observer)
	}
}
