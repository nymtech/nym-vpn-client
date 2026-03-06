package net.nymtech.nymvpn.data.config

import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import dagger.hilt.android.qualifiers.ApplicationContext
import jakarta.inject.Singleton
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpn.model.config.CoreAppConfigProvider
import nym_vpn_lib_types.UserAgent
import javax.inject.Inject

@Singleton
class AppConfigProvider @Inject constructor(@ApplicationContext private val context: Context) : CoreAppConfigProvider {

	override fun getUserAgent(): UserAgent {
		val platform = if (isAndroidTV()) "AndroidTV" else "Android"
		return UserAgent(
			application = Constants.APP_PROJECT_NAME,
			version = BuildConfig.VERSION_NAME,
			platform = "$platform; ${Build.VERSION.SDK_INT}; ${NymVpn.Companion.getCPUArchitecture()}; ${BuildConfig.FLAVOR}",
			gitCommit = BuildConfig.COMMIT_HASH,
		)
	}

	private fun isAndroidTV(): Boolean = context.packageManager.hasSystemFeature(PackageManager.FEATURE_LEANBACK)
}
