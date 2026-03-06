package net.nymtech.nymvpn.di

import android.content.Context
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.SplitTunnelingRepository
import net.nymtech.nymvpn.data.config.BackedVpnConfigRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.data.datastore.DataStoreGatewayRepository
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.data.datastore.DataStoreSettingsRepository
import net.nymtech.nymvpn.data.datastore.DataStoreSplitTunnelingRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.manager.backend.VpnServiceConnectionManager
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
	fun provideGatewayRepository(dataStoreManager: DataStoreManager): GatewayRepository = DataStoreGatewayRepository(dataStoreManager)

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
