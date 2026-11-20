package net.nymtech.nymvpn.di

import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.backend.ServiceBackedBackendManager
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class BackendModule {

	@Binds
	@Singleton
	abstract fun bindBackendManager(impl: ServiceBackedBackendManager): BackendManager
}
