package net.nymtech.nymvpn.di

import android.content.Context
import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import net.nymtech.nymvpn.manager.backend.NymBackendManager
import net.nymtech.nymvpn.manager.backend.BackendManager
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class ManagerModule {

	@Binds
	@Singleton
	abstract fun bindContext(@ApplicationContext context: Context): Context

	@Binds
	@Singleton
	abstract fun bindNymVpnManager(nymVpnManager: NymBackendManager): BackendManager
}
