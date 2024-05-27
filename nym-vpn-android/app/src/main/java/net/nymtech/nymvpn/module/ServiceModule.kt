package net.nymtech.nymvpn.module

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ServiceComponent
import dagger.hilt.android.scopes.ServiceScoped
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob

@Module
@InstallIn(ServiceComponent::class)
class ServiceModule {

	@Provides
	@ServiceScope
	@ServiceScoped
	fun providesApplicationScope(@IoDispatcher ioDispatcher: CoroutineDispatcher): CoroutineScope = CoroutineScope(SupervisorJob() + ioDispatcher)
}
