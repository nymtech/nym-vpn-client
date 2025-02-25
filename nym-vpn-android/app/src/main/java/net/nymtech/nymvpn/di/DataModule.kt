package net.nymtech.nymvpn.di

import android.content.Context
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.datastore.DataStoreGatewayRepository
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.data.datastore.DataStoreSettingsRepository
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
class DataModule {
	@Singleton
	@Provides
	fun providePreferencesDataStore(@ApplicationContext context: Context, @IoDispatcher dispatcher: CoroutineDispatcher): DataStoreManager {
		return DataStoreManager(context, dispatcher)
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
