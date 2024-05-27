package net.nymtech.nymvpn.module

import android.content.Context
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SecretsRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.datastore.DataStoreGatewayRepository
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.data.datastore.DataStoreSettingsRepository
import net.nymtech.nymvpn.data.datastore.EncryptedPreferences
import net.nymtech.nymvpn.data.datastore.SecretsPreferencesRepository
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

	@Singleton
	@Provides
	fun provideEncryptedPreferences(@ApplicationContext context: Context): EncryptedPreferences {
		return EncryptedPreferences(context)
	}

	@Singleton
	@Provides
	fun provideSecretsRepository(encryptedPreferences: EncryptedPreferences, @IoDispatcher dispatcher: CoroutineDispatcher): SecretsRepository {
		return SecretsPreferencesRepository(encryptedPreferences, dispatcher)
	}
}
