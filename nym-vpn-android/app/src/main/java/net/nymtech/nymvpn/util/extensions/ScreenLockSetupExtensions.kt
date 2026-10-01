package net.nymtech.nymvpn.util.extensions

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.provider.Settings
import androidx.activity.result.ActivityResultLauncher
import timber.log.Timber

fun ActivityResultLauncher<Intent>.launchScreenLockSetup(setupIntent: Intent) = launchWithSecuritySettingsFallback(setupIntent) { launch(it) }

fun Context.launchScreenLockSetup(setupIntent: Intent) = launchWithSecuritySettingsFallback(setupIntent) { startActivity(it) }

// Some OEMs don't handle ACTION_BIOMETRIC_ENROLL, so fall back to the generic security settings.
private inline fun launchWithSecuritySettingsFallback(setupIntent: Intent, launch: (Intent) -> Unit) {
	try {
		launch(setupIntent)
	} catch (e: ActivityNotFoundException) {
		Timber.w(e, "Screen lock setup intent not handled, falling back to security settings")
		launch(Intent(Settings.ACTION_SECURITY_SETTINGS))
	}
}
