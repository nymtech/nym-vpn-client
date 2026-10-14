package net.nymtech.nymvpn.di

import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import net.nymtech.nymvpn.manager.deviceauth.NymDeviceAuthManager
import net.nymtech.nymvpn.manager.deviceauth.DeviceAuthManager
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class DeviceAuthModule {

	@Binds
	@Singleton
	abstract fun bindDeviceAuthManager(impl: NymDeviceAuthManager): DeviceAuthManager
}
