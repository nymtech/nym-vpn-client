package net.nymtech.nymvpn.di

import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import net.nymtech.nymvpn.manager.freepass.FreepassManager
import net.nymtech.nymvpn.manager.freepass.NymFreepassManager
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class FreepassModule {

	@Binds
	@Singleton
	abstract fun bindFreepassManager(impl: NymFreepassManager): FreepassManager
}
