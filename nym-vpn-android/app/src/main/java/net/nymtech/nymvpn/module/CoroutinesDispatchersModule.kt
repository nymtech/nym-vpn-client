package net.nymtech.nymvpn.module

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import net.nymtech.nymvpn.module.qualifiers.DefaultDispatcher
import net.nymtech.nymvpn.module.qualifiers.IoDispatcher
import net.nymtech.nymvpn.module.qualifiers.MainDispatcher
import net.nymtech.nymvpn.module.qualifiers.MainImmediateDispatcher

@InstallIn(SingletonComponent::class)
@Module
object CoroutinesDispatchersModule {

	@DefaultDispatcher
	@Provides
	fun providesDefaultDispatcher(): CoroutineDispatcher = Dispatchers.Default

	@IoDispatcher
	@Provides
	fun providesIoDispatcher(): CoroutineDispatcher = Dispatchers.IO

	@MainDispatcher
	@Provides
	fun providesMainDispatcher(): CoroutineDispatcher = Dispatchers.Main

	@MainImmediateDispatcher
	@Provides
	fun providesMainImmediateDispatcher(): CoroutineDispatcher = Dispatchers.Main.immediate
}
