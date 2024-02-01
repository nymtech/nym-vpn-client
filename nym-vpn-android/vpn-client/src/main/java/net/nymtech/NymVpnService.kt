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
    private lateinit var vpnInterface: ParcelFileDescriptor
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
                // Create a new VPN Builder
                val builder = Builder()

                // Set the VPN parameters
                builder.setSession("nymtun")
                    .addAddress("10.0.0.1", 24)
                    .addRoute("0.0.0.0", 1)
                    .addRoute("128.0.0.0", 1)
                    .addRoute("8000::", 1)
                    .addRoute("::", 1)
//                    .addDnsServer("8.8.8.8")
//                    .addRoute("0.0.0.0", 0)
//                    .setMtu(1500)


                // Establish the VPN connection
                builder.establish()?.let {
                    vpnInterface = it
                    Timber.d("Interface created")
                    start(vpnInterface.fd)
                }

                // Redirect network traffic through the VPN interface
//                val vpnInput = FileInputStream(vpnInterface.fileDescriptor)
//                val vpnOutput = FileOutputStream(vpnInterface.fileDescriptor)

//                while (true) {
//                    // Read incoming network traffic from vpnInput
//                    // Process the traffic as needed
//
//                    // Write outgoing network traffic to vpnOutput
//                    // Send the traffic through the VPN interface
//                }
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
            vpnInterface.close()
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        stopVpn()
    }

    private fun start(interfaceFd : Int) {
        vpnClient.connect("FR", "FR", this, interfaceFd)
    }

    private fun stop() {
        stopVpn()
        stopSelf()
    }
}