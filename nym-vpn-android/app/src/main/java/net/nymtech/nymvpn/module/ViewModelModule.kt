package net.nymtech.nymvpn.module

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ViewModelComponent
import dagger.hilt.android.scopes.ViewModelScoped
import net.nymtech.nymvpn.NymVpn
import net.nymtech.vpn.NymApi

@Module
@InstallIn(ViewModelComponent::class)
internal object ViewModelModule {
	@Provides
	@ViewModelScoped
	fun provideNymApi(): NymApi {
		return NymApi(NymVpn.environment)
	}
}
