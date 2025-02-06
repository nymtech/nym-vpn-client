package net.nymtech.nymvpn.module

import android.content.Context
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import net.nymtech.connectivity.NetworkConnectivityService
import net.nymtech.connectivity.NetworkService
import net.nymtech.logcatutil.LogReader
import net.nymtech.logcatutil.LogcatReader
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.shortcut.DynamicShortcutManager
import net.nymtech.nymvpn.manager.shortcut.ShortcutManager
import net.nymtech.nymvpn.module.qualifiers.ApplicationScope
import net.nymtech.nymvpn.module.qualifiers.DefaultDispatcher
import net.nymtech.nymvpn.module.qualifiers.IoDispatcher
import net.nymtech.nymvpn.service.gateway.GatewayCacheService
import net.nymtech.nymvpn.service.gateway.GatewayDataStoreCacheService
import net.nymtech.nymvpn.service.notification.NotificationService
import net.nymtech.nymvpn.service.notification.VpnAlertNotifications
import net.nymtech.nymvpn.util.FileUtils
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
	fun provideCountryCacheService(backendManager: BackendManager, gatewayRepository: GatewayRepository): GatewayCacheService {
		return GatewayDataStoreCacheService(gatewayRepository, backendManager)
	}

	@Singleton
	@Provides
	fun provideLogcatHelper(@ApplicationContext context: Context): LogReader {
		return LogcatReader.init(storageDir = context.filesDir.absolutePath)
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

	@Singleton
	@Provides
	fun networkConnectivityService(@ApplicationContext context: Context): NetworkService {
		return NetworkConnectivityService(context)
	}
}
