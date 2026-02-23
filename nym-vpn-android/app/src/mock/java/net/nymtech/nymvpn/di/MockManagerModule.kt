package net.nymtech.nymvpn.di

import android.content.Context
import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.backend.MockBackendManager
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.manager.billing.NymBillingManager
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import net.nymtech.nymvpn.manager.environment.NymEnvironmentManager
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class ManagerModule {

	@Binds
	@Singleton
	abstract fun bindContext(@ApplicationContext context: Context): Context

	@Binds
	@Singleton
	abstract fun bindBackendManager(impl: MockBackendManager): BackendManager

	@Binds
	@Singleton
	abstract fun bindNymEnvironmentManager(environmentManager: NymEnvironmentManager): EnvironmentManager

	@Binds
	@Singleton
	abstract fun bindNymBillingManager(billingManager: NymBillingManager): BillingManager
}
