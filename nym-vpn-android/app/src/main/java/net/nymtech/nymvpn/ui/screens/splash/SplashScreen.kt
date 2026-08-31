package net.nymtech.nymvpn.ui.screens.splash

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.util.extensions.navigateAndForget

@Composable
fun SplashScreen(appViewModel: AppViewModel, appUiState: AppUiState, topOffset: Dp = 0.dp) {
	val navController = LocalNavController.current
	val isAppReady by appViewModel.isAppReady.collectAsStateWithLifecycle()

	LaunchedEffect(appUiState.managerState.isInitialized, isAppReady) {
		if (appUiState.managerState.isInitialized && isAppReady) {
			val shouldShowOnboarding = !appUiState.settings.isOnboardingCompleted && !appUiState.managerState.isMnemonicStored
			val destination = if (shouldShowOnboarding) Route.Onboarding else Route.Main()
			navController.navigateAndForget(destination)
		}
	}

	val infiniteTransition = rememberInfiniteTransition(label = "SplashTransition")

	val scale by infiniteTransition.animateFloat(
		initialValue = 0.995f,
		targetValue = 1.05f,
		animationSpec = infiniteRepeatable(
			animation = tween(durationMillis = 1000, easing = FastOutSlowInEasing),
			repeatMode = RepeatMode.Reverse,
		),
		label = "PulseAnimation",
	)

	val primaryColor = MaterialTheme.colorScheme.tertiary

	Box(
		contentAlignment = Alignment.Center,
		modifier = Modifier
			.fillMaxSize()
			.background(MaterialTheme.colorScheme.background)
			.padding(bottom = topOffset),
	) {
		Icon(
			imageVector = ImageVector.vectorResource(R.drawable.app_label),
			contentDescription = stringResource(R.string.app_name),
			tint = primaryColor,
			modifier = Modifier
				.size(140.dp)
				.graphicsLayer {
					scaleX = scale
					scaleY = scale
				},
		)
	}
}
