package net.nymtech.nymvpn.di

import android.content.Context
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.SplitTunnelingRepository
import net.nymtech.nymvpn.data.config.BackedVpnConfigRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.data.datastore.LegacyGatewayPreferences
import net.nymtech.nymvpn.data.file.FileGatewayRepository
import net.nymtech.nymvpn.data.datastore.DataStoreSettingsRepository
import net.nymtech.nymvpn.data.datastore.DataStoreSplitTunnelingRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.manager.backend.VpnServiceConnectionManager
import java.io.File
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
class DataModule {
	@Singleton
	@Provides
	fun providePreferencesDataStore(@ApplicationContext context: Context, @IoDispatcher dispatcher: CoroutineDispatcher): DataStoreManager = DataStoreManager(context, dispatcher)

	@Singleton
	@Provides
	fun provideSettingsRepository(dataStoreManager: DataStoreManager): SettingsRepository = DataStoreSettingsRepository(dataStoreManager)

	@Singleton
	@Provides
	fun provideGatewayRepository(
		@ApplicationContext context: Context,
		dataStoreManager: DataStoreManager,
		@ApplicationScope appScope: CoroutineScope,
		@IoDispatcher ioDispatcher: CoroutineDispatcher,
	): GatewayRepository {
		val repository = FileGatewayRepository(File(context.filesDir, FileGatewayRepository.DIRECTORY_NAME), ioDispatcher)
		// Gateway lists used to be JSON strings inside the shared preferences file; move them over so that file shrinks.
		appScope.launch { LegacyGatewayPreferences.migrate(dataStoreManager, repository) }
		return repository
	}

	@Singleton
	@Provides
	fun provideSplitTunnelingRepository(dataStoreManager: DataStoreManager): SplitTunnelingRepository = DataStoreSplitTunnelingRepository(dataStoreManager)

	@Singleton
	@Provides
	fun provideVpnConfigRepository(
		serviceConnectionManager: VpnServiceConnectionManager,
		@ApplicationScope appScope: CoroutineScope,
		@IoDispatcher ioDispatcher: CoroutineDispatcher,
	): VpnConfigRepository = BackedVpnConfigRepository(
		serviceConnectionManager = serviceConnectionManager,
		appScope = appScope,
		ioDispatcher = ioDispatcher,
	)
}
