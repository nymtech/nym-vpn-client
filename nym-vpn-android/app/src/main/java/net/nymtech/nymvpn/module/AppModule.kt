package net.nymtech.nymvpn.module

import android.content.Context
import android.os.Build
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import net.nymtech.logcatutil.LogCollect
import net.nymtech.logcatutil.LogcatHelper
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.manager.shortcut.DynamicShortcutManager
import net.nymtech.nymvpn.manager.shortcut.ShortcutManager
import net.nymtech.nymvpn.module.qualifiers.ApplicationScope
import net.nymtech.nymvpn.module.qualifiers.DefaultDispatcher
import net.nymtech.nymvpn.module.qualifiers.IoDispatcher
import net.nymtech.nymvpn.service.country.CountryCacheService
import net.nymtech.nymvpn.service.country.CountryDataStoreCacheService
import net.nymtech.nymvpn.service.gateway.NymApiLibService
import net.nymtech.nymvpn.service.gateway.NymApiService
import net.nymtech.nymvpn.service.notification.NotificationService
import net.nymtech.nymvpn.service.notification.VpnAlertNotifications
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.FileUtils
import net.nymtech.nymvpn.util.extensions.isAndroidTV
import net.nymtech.vpn.NymApi
import net.nymtech.vpn.backend.Backend
import net.nymtech.vpn.backend.NymBackend
import nym_vpn_lib.UserAgent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AppModule {

	@Singleton
	@ApplicationScope
	@Provides
	fun providesApplicationScope(@DefaultDispatcher defaultDispatcher: CoroutineDispatcher): CoroutineScope =
		CoroutineScope(SupervisorJob() + defaultDispatcher)

	@Singleton
	@Provides
	fun provideNymApi(@IoDispatcher dispatcher: CoroutineDispatcher, @ApplicationContext context: Context): NymApi {
		val platform = if (context.isAndroidTV()) "AndroidTV" else "Android"
		return NymApi(
			dispatcher,
			UserAgent(
				Constants.APP_PROJECT_NAME,
				BuildConfig.VERSION_NAME,
				"$platform; ${Build.VERSION.SDK_INT}; ${NymVpn.getCPUArchitecture()}; ${BuildConfig.FLAVOR}",
				BuildConfig.COMMIT_HASH,
			),
		)
	}

	@Singleton
	@Provides
	fun provideGatewayLibService(nymApi: NymApi): NymApiService {
		return NymApiLibService(nymApi)
	}

	@Singleton
	@Provides
	fun provideCountryCacheService(nymApiService: NymApiService, gatewayRepository: GatewayRepository): CountryCacheService {
		return CountryDataStoreCacheService(gatewayRepository, nymApiService)
	}

	@Singleton
	@Provides
	fun provideBackend(@ApplicationContext context: Context): Backend {
		return NymBackend.getInstance(context)
	}

	@Singleton
	@Provides
	fun provideLogcatHelper(@ApplicationContext context: Context): LogCollect {
		return LogcatHelper.init(context = context)
	}

	@Singleton
	@Provides
	fun provideFileUtils(@ApplicationContext context: Context, @IoDispatcher dispatcher: CoroutineDispatcher): FileUtils {
		return FileUtils(context, dispatcher)
	}

	@Singleton
	@Provides
	fun provideNotificationService(@ApplicationContext context: Context): NotificationService {
		return VpnAlertNotifications(context)
	}

	@Singleton
	@Provides
	fun provideShortcutManager(@ApplicationContext context: Context): ShortcutManager {
		return DynamicShortcutManager(context)
	}
}
