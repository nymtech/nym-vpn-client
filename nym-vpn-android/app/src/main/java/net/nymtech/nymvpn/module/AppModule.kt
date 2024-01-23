package net.nymtech.nymvpn.module

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import net.nymtech.vpn_client.NymVpnClient
import net.nymtech.vpn_client.VpnClient
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