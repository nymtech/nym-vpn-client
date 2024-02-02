package net.nymtech

import android.app.Notification
import android.content.Intent
import android.net.VpnService
import android.os.ParcelFileDescriptor
import androidx.core.app.NotificationCompat
import net.nymtech.uniffi.lib.nymVPNLib
import net.nymtech.vpn_client.Action
import net.nymtech.vpn_client.NymVpnClient
import net.nymtech.vpn_client.VpnClient
import timber.log.Timber

class NymVpnService : VpnService() {

    private var vpnClient : VpnClient = NymVpnClient()
    private lateinit var vpnThread: Thread
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Timber.d("On start received")
        return if (intent?.action == Action.START.name) {

            Timber.d("VPN start intent")
            startVpn()
            START_STICKY
        } else {
            Timber.d("VPN stopping intent")
            //stop()
            START_NOT_STICKY
        }
    }

    private fun startVpn() {
        vpnThread = Thread {
            try {
                start()
            } catch (e: Exception) {
                // Handle VPN connection errors
                e.printStackTrace()
            } finally {
                //stopVpn()
            }
        }

        vpnThread.start()
    }

    private fun stopVpn() {
        try {
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        stopVpn()
    }

    private fun start() {
        vpnClient.connect("FR", "FR", this)
    }

    private fun stop() {
        stopVpn()
        stopSelf()
    }
}