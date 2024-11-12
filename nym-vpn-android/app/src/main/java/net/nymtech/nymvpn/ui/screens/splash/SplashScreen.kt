package net.nymtech.nymvpn.ui.screens.splash

import androidx.compose.foundation.background
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.airbnb.lottie.compose.LottieAnimation
import com.airbnb.lottie.compose.LottieCompositionSpec
import com.airbnb.lottie.compose.animateLottieCompositionAsState
import com.airbnb.lottie.compose.rememberLottieComposition
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.navigateAndForget

@Composable
fun SplashScreen(appViewModel: AppViewModel, appUiState: AppUiState) {
	val navController = LocalNavController.current

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				show = false,
			),
		)
	}

	Box(
		modifier = Modifier
			.fillMaxSize()
			.background(MaterialTheme.colorScheme.background),
	) {
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.Center,
			modifier = Modifier.fillMaxSize(),
		) {
			val animation = when (appUiState.settings.theme) {
				Theme.DARK_MODE -> R.raw.splash_animation_dark
				Theme.LIGHT_MODE -> R.raw.splash_animation_light
				else -> if (isSystemInDarkTheme()) {
					R.raw.splash_animation_dark
				} else {
					R.raw.splash_animation_light
				}
			}
			val composition = rememberLottieComposition(LottieCompositionSpec.RawRes(animation))
			val logoAnimationState =
				animateLottieCompositionAsState(composition = composition.value, speed = 2.0f)
			LottieAnimation(
				composition = composition.value,
				progress = { logoAnimationState.progress },
			)
			if (logoAnimationState.isAtEnd && logoAnimationState.isPlaying) {
				// Ignore analytics screen for now
// 				val route = if (appUiState.settings.isAnalyticsShown) {
// 					Route.Main()
// 				} else {
// 					Route.Analytics
// 				}
				navController.navigateAndForget(Route.Main())
			}
		}
	}
}
