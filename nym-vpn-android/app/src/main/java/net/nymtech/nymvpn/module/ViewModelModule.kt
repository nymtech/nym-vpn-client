package net.nymtech.nymvpn.module

import android.content.Context
import androidx.navigation.NavHostController
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ViewModelComponent
import dagger.hilt.android.qualifiers.ApplicationContext
import net.nymtech.nymvpn.ui.common.navigation.NavigationService

@Module
@InstallIn(ViewModelComponent::class)
object NavigationModule {
	@Provides
	fun provideNestedNavController(@ApplicationContext context: Context): NavHostController {
		return NavigationService(context).navController}
}
