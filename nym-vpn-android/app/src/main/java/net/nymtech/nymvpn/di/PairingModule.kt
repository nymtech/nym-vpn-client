package net.nymtech.nymvpn.di

import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import net.nymtech.nymvpn.manager.pairing.NymPairingManager
import net.nymtech.nymvpn.manager.pairing.PairingManager
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class PairingModule {

	@Binds
	@Singleton
	abstract fun bindPairingManager(impl: NymPairingManager): PairingManager
}
