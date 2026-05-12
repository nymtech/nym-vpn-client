package net.nymtech.nymvpn.ui.screens.splash

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.util.extensions.navigateAndForget

@Composable
fun SplashScreen(appViewModel: AppViewModel, appUiState: AppUiState) {
	val navController = LocalNavController.current
	val isAppReady by appViewModel.isAppReady.collectAsStateWithLifecycle()

	LaunchedEffect(appUiState.managerState.isInitialized, isAppReady) {
		if (appUiState.managerState.isInitialized && isAppReady) {
			navController.navigateAndForget(Route.Main())
		}
	}

	Box(
		contentAlignment = Alignment.Center,
		modifier = Modifier
			.fillMaxSize()
			.background(MaterialTheme.colorScheme.background),
	) {
		Icon(
			imageVector = ImageVector.vectorResource(R.drawable.app_label),
			contentDescription = null,
			tint = MaterialTheme.colorScheme.onBackground,
		)
	}
}
