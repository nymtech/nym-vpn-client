package net.nymtech.nymvpn.di

import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import net.nymtech.nymvpn.data.config.AppConfigProvider
import net.nymtech.vpn.model.config.CoreAppConfigProvider
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class ConfigModule {
	@Binds
	@Singleton
	abstract fun bindAppConfigProvider(impl: AppConfigProvider): CoreAppConfigProvider
}
