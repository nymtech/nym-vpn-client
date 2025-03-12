package net.nymtech.nymvpn.di

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import net.nymtech.nymvpn.di.qualifiers.DefaultDispatcher
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.di.qualifiers.MainDispatcher
import net.nymtech.nymvpn.di.qualifiers.MainImmediateDispatcher

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
