package net.nymtech.nymvpn.module

import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import net.nymtech.nymvpn.service.vpn.NymVpnManager
import net.nymtech.nymvpn.service.vpn.VpnManager
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class ManagerModule {
	@Binds
	@Singleton
	abstract fun bindNymVpnManager(nymVpnManager: NymVpnManager): VpnManager
}
