package net.nymtech.nymvpn.module

import android.content.Context
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.datastore.DataStoreGatewayRepository
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.data.datastore.DataStoreSettingsRepository
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
class DataModule {
	@Singleton
	@Provides
	fun providePreferencesDataStore(@ApplicationContext context: Context): DataStoreManager {
		return DataStoreManager(context)
	}

	@Singleton
	@Provides
	fun provideSettingsRepository(dataStoreManager: DataStoreManager): SettingsRepository {
		return DataStoreSettingsRepository(dataStoreManager)
	}

	@Singleton
	@Provides
	fun provideGatewayRepository(dataStoreManager: DataStoreManager): GatewayRepository {
		return DataStoreGatewayRepository(dataStoreManager)
	}
}
