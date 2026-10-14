package net.nymtech.nymvpn.manager.deviceauth

import android.content.Intent
import androidx.fragment.app.FragmentActivity
import net.nymtech.nymvpn.manager.deviceauth.model.DeviceAuthResult

interface DeviceAuthManager {
	fun isAuthSetUp(): Boolean
	suspend fun authenticate(activity: FragmentActivity, title: String, subtitle: String): DeviceAuthResult
	fun createSetupIntent(): Intent
}
