package net.nymtech.nymvpn.module

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ViewModelComponent
import dagger.hilt.android.scopes.ViewModelScoped
import net.nymtech.vpn.NymApi
import net.nymtech.nymvpn.NymVpn

@Module
@InstallIn(ViewModelComponent::class)
internal object ViewModelModule {
	@Provides
	@ViewModelScoped
	fun provideNymApi(): NymApi {
		return NymApi(NymVpn.environment)
	}
}
