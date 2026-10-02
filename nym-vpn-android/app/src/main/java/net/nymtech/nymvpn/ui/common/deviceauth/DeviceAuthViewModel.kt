package net.nymtech.nymvpn.ui.common.deviceauth

import android.content.Intent
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.nymtech.nymvpn.manager.deviceauth.DeviceAuthManager
import net.nymtech.nymvpn.manager.deviceauth.model.DeviceAuthResult
import javax.inject.Inject

@HiltViewModel
class DeviceAuthViewModel @Inject constructor(private val deviceAuthManager: DeviceAuthManager) : ViewModel() {

	fun isScreenLockSet(): Boolean = deviceAuthManager.isAuthSetUp()

	suspend fun authenticate(activity: FragmentActivity, title: String, subtitle: String): DeviceAuthResult = deviceAuthManager.authenticate(activity, title, subtitle)

	fun createScreenLockSetupIntent(): Intent = deviceAuthManager.createSetupIntent()
}
