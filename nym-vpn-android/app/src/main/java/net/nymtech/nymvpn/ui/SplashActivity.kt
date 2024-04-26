package net.nymtech.nymvpn.ui

import android.annotation.SuppressLint
import android.content.Intent
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import dagger.hilt.android.AndroidEntryPoint
import io.sentry.android.core.SentryAndroid
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.country.CountryCacheService
import net.nymtech.nymvpn.util.Constants
import timber.log.Timber
import javax.inject.Inject

@SuppressLint("CustomSplashScreen")
@AndroidEntryPoint
class SplashActivity : ComponentActivity() {

	@Inject
	lateinit var countryCacheService: CountryCacheService

	@Inject
	lateinit var settingsRepository: SettingsRepository

	override fun onCreate(savedInstanceState: Bundle?) {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
			val splashScreen = installSplashScreen()
			splashScreen.setKeepOnScreenCondition { true }
		}
		super.onCreate(savedInstanceState)
		lifecycleScope.launch(Dispatchers.IO) {
			repeatOnLifecycle(Lifecycle.State.CREATED) {
				// init data
				settingsRepository.init()

				NymVpn.applicationScope.launch(Dispatchers.IO) {
					listOf(
						async {
							Timber.d("Updating exit country cache")
							countryCacheService.updateExitCountriesCache()
							Timber.d("Exit countries updated")
						},
						async {
							Timber.d("Updating entry country cache")
							countryCacheService.updateEntryCountriesCache()
							Timber.d("Entry countries updated")
						},
						async {
							Timber.d("Updating low latency country cache")
							countryCacheService.updateLowLatencyEntryCountryCache()
							Timber.d("Low latency country updated")
						},
					).awaitAll()
				}

				configureSentry()

				val isAnalyticsShown = settingsRepository.isAnalyticsShown()

				val intent = Intent(this@SplashActivity, MainActivity::class.java).apply {
					putExtra(IS_ANALYTICS_SHOWN_INTENT_KEY, isAnalyticsShown)
				}
				startActivity(intent)
				finish()
			}
		}
	}

	private suspend fun configureSentry() {
		if (settingsRepository.isErrorReportingEnabled()) {
			SentryAndroid.init(this@SplashActivity) { options ->
				options.enableTracing = true
				options.enableAllAutoBreadcrumbs(true)
				options.isEnableUserInteractionTracing = true
				options.isEnableUserInteractionBreadcrumbs = true
				options.dsn = BuildConfig.SENTRY_DSN
				options.sampleRate = 1.0
				options.tracesSampleRate = 1.0
				options.profilesSampleRate = 1.0
				options.environment =
					if (BuildConfig.DEBUG) Constants.SENTRY_DEV_ENV else Constants.SENTRY_PROD_ENV
			}
		}
	}
	companion object {
		const val IS_ANALYTICS_SHOWN_INTENT_KEY = "is_analytics_shown"
	}
}
