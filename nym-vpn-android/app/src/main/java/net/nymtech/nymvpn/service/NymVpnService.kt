package net.nymtech.nymvpn.service

import android.content.Intent
import android.net.VpnService
import android.os.IBinder
import dagger.hilt.android.AndroidEntryPoint


@AndroidEntryPoint
class NymVpnService : VpnService() {

    override fun onCreate() {
        super.onCreate()
    }

    override fun onBind(intent: Intent?): IBinder? {
        return null
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        return START_STICKY
    }
}