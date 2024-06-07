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
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.logcathelper.LogCollect
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.ApplicationScope
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

	@Inject
	lateinit var logCollect: LogCollect

	@Inject
	@ApplicationScope
	lateinit var applicationScope: CoroutineScope

	override fun onCreate(savedInstanceState: Bundle?) {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
			val splashScreen = installSplashScreen()
			splashScreen.setKeepOnScreenCondition { true }
		}
		super.onCreate(savedInstanceState)

		applicationScope.launch {
			launch {
				Timber.d("Updating exit country cache")
				countryCacheService.updateExitCountriesCache().onSuccess {
					Timber.d("Exit countries updated")
				}.onFailure { Timber.w("Failed to get exit countries: ${it.message}") }
			}
			launch {
				Timber.d("Updating entry country cache")
				countryCacheService.updateEntryCountriesCache().onSuccess {
					Timber.d("Entry countries updated")
				}.onFailure { Timber.w("Failed to get entry countries: ${it.message}") }
			}
		}

		lifecycleScope.launch {
			repeatOnLifecycle(Lifecycle.State.CREATED) {
				// init data
				settingsRepository.init()

				configureSentry()

				val isAnalyticsShown = settingsRepository.isAnalyticsShown()
				val theme = settingsRepository.getTheme()

				val intent = Intent(this@SplashActivity, MainActivity::class.java).apply {
					putExtra(IS_ANALYTICS_SHOWN_INTENT_KEY, isAnalyticsShown)
					putExtra(THEME, theme.name)
				}
				startActivity(intent)
				finish()
			}
		}
	}

	private suspend fun configureSentry() {
		if (settingsRepository.isErrorReportingEnabled()) {
			SentryAndroid.init(NymVpn.instance) { options ->
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
		const val THEME = "theme"
	}
}
