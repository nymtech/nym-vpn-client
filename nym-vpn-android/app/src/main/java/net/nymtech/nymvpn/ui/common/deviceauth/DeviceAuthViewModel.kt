package net.nymtech.nymvpn.ui.common.deviceauth

import android.content.Intent
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.nymtech.nymvpn.manager.deviceauth.DeviceAuthGate
import net.nymtech.nymvpn.manager.deviceauth.DeviceAuthManager
import net.nymtech.nymvpn.manager.deviceauth.model.DeviceAuthResult
import timber.log.Timber
import java.security.GeneralSecurityException
import java.security.ProviderException
import javax.inject.Inject

@HiltViewModel
class DeviceAuthViewModel @Inject constructor(private val deviceAuthManager: DeviceAuthManager, private val deviceAuthGate: DeviceAuthGate) : ViewModel() {

	fun isScreenLockSet(): Boolean = deviceAuthManager.isAuthSetUp()

	suspend fun authenticate(activity: FragmentActivity, title: String, subtitle: String, keyBound: Boolean): DeviceAuthResult {
		if (!keyBound) return deviceAuthManager.authenticate(activity, title, subtitle)
		val cryptoObject = try {
			deviceAuthGate.createCryptoObject()
		} catch (e: GeneralSecurityException) {
			Timber.tag(TAG).w(e, "GateKeyUnavailable")
			return DeviceAuthResult.Error(DeviceAuthResult.Error.KEY_UNAVAILABLE)
		} catch (e: ProviderException) {
			Timber.tag(TAG).w(e, "GateKeyUnavailable")
			return DeviceAuthResult.Error(DeviceAuthResult.Error.KEY_UNAVAILABLE)
		}
		return deviceAuthManager.authenticateStrong(activity, title, subtitle, cryptoObject)
	}

	fun createScreenLockSetupIntent(): Intent = deviceAuthManager.createSetupIntent()

	private companion object {
		const val TAG = "device-auth"
	}
}
