package net.nymtech.nymvpn.di

import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.manager.billing.NymBillingManager
import net.nymtech.nymvpn.manager.deviceauth.DeviceAuthManager
import net.nymtech.nymvpn.manager.deviceauth.NymDeviceAuthManager
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import net.nymtech.nymvpn.manager.environment.NymEnvironmentManager
import net.nymtech.nymvpn.manager.favorites.FavoritesManager
import net.nymtech.nymvpn.manager.favorites.NymFavoritesManager
import net.nymtech.nymvpn.manager.freepass.FreepassManager
import net.nymtech.nymvpn.manager.freepass.NymFreepassManager
import net.nymtech.nymvpn.manager.pairing.NymPairingManager
import net.nymtech.nymvpn.manager.pairing.PairingManager
import javax.inject.Singleton

// BackendManager is bound per flavor in BackendModule.
@Module
@InstallIn(SingletonComponent::class)
abstract class ManagerModule {

	@Binds
	@Singleton
	abstract fun bindNymEnvironmentManager(environmentManager: NymEnvironmentManager): EnvironmentManager

	@Binds
	@Singleton
	abstract fun bindNymBillingManager(billingManager: NymBillingManager): BillingManager

	@Binds
	@Singleton
	abstract fun bindNymFavoritesManager(favoritesManager: NymFavoritesManager): FavoritesManager

	@Binds
	@Singleton
	abstract fun bindDeviceAuthManager(impl: NymDeviceAuthManager): DeviceAuthManager

	@Binds
	@Singleton
	abstract fun bindFreepassManager(impl: NymFreepassManager): FreepassManager

	@Binds
	@Singleton
	abstract fun bindPairingManager(impl: NymPairingManager): PairingManager
}
