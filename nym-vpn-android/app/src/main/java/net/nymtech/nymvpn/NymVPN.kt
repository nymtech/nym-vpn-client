package net.nymtech.nymvpn

import android.app.Application
import android.net.VpnService
import dagger.hilt.android.HiltAndroidApp
import net.nymtech.NymVpnService
import net.nymtech.vpn_client.NymVpnClient
import timber.log.Timber


@HiltAndroidApp
class NymVPN : Application() {

    override fun onCreate() {
        super.onCreate()
        instance = this
        if (BuildConfig.DEBUG) Timber.plant(Timber.DebugTree())
    }

    companion object {
        lateinit var instance : NymVPN
            private set
    }
}