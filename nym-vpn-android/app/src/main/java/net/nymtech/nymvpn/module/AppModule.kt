package net.nymtech.nymvpn.module

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import net.nymtech.vpn.NymVpnClient
import net.nymtech.vpn.VpnClient
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
class AppModule {

    @Singleton
    @Provides
    fun provideVpnClient(): VpnClient {
        return NymVpnClient()
    }
}