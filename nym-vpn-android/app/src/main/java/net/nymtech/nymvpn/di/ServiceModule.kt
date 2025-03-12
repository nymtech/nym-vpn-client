package net.nymtech.nymvpn.di

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ServiceComponent
import dagger.hilt.android.scopes.ServiceScoped
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.di.qualifiers.ServiceScope

@Module
@InstallIn(ServiceComponent::class)
class ServiceModule {

	@Provides
	@ServiceScope
	@ServiceScoped
	fun providesApplicationScope(@IoDispatcher ioDispatcher: CoroutineDispatcher): CoroutineScope = CoroutineScope(SupervisorJob() + ioDispatcher)
}
