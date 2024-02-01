package net.nymtech.uniffi.lib

import android.net.VpnService
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.nymtech.NymVpnService
import timber.log.Timber

const val nymVPNLib = "nym_vpn_lib"

class NymVPN {
    // Load the native library "libnym_vpn_lib.so"
    init {
        System.loadLibrary(nymVPNLib)
        Timber.i( "loaded native library $nymVPNLib")
    }

    fun init(api_url: String,
             entry_gateway: String,
             exit_router: String,
             vpn_service: Any,
             interface_fd: Int) {
        Timber.d("calling $nymVPNLib:initVPN")
        try {
            initVPN(api_url, entry_gateway, exit_router, vpn_service, interface_fd)
        } catch (e: Throwable) {
            Timber.i("$nymVPNLib:initVPN internal error: $e")
            //Sentry.captureException(e)
        }
    }
    fun run() {
        Timber.d("calling lib:runVPN")
        try {
            runVPN()
        } catch (e: Throwable) {
            Timber.e("lib:runVPN internal error: $e")
            //Sentry.captureException(e)
        }
    }

    fun stop() {
        Timber.e( "calling lib:stopVPN")
        try {
            stopVPN()
        } catch (e: Throwable) {
             Timber.e("lib:stopVPN internal error: $e")
            //Sentry.captureException(e)
        }
    }

    private external fun initVPN(
        api_url: String,
        entry_gateway: String,
        exit_router: String,
        vpn_service: Any,
        interface_fd: Int
    )
    private external fun runVPN()
    private external fun stopVPN()
}