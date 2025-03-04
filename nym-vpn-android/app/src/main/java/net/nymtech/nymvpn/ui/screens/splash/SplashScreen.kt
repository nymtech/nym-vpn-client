package net.nymtech.nymvpn.ui.screens.splash

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.airbnb.lottie.compose.LottieAnimation
import com.airbnb.lottie.compose.LottieCancellationBehavior
import com.airbnb.lottie.compose.LottieCompositionSpec
import com.airbnb.lottie.compose.animateLottieCompositionAsState
import com.airbnb.lottie.compose.rememberLottieComposition
import kotlinx.coroutines.delay
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.theme.ThemeColors
import net.nymtech.nymvpn.util.extensions.navigateAndForget

@Composable
fun SplashScreen(appViewModel: AppViewModel, appUiState: AppUiState) {
	val navController = LocalNavController.current

	val composition = rememberLottieComposition(LottieCompositionSpec.RawRes(R.raw.intro_logo))
	var splashFinished by remember { mutableStateOf(false) }
	val isAppReady by appViewModel.isAppReady.collectAsStateWithLifecycle()

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				show = false,
			),
		)
		appViewModel.onAppStartup()
	}

	LaunchedEffect(composition) {
		delay(3000)
		splashFinished = true
	}

	LaunchedEffect(splashFinished, appUiState.managerState.isInitialized, isAppReady) {
		if (splashFinished && appUiState.managerState.isInitialized && isAppReady) {
			navController.navigateAndForget(Route.Main())
		}
	}

	Box(
		modifier = Modifier
			.fillMaxSize()
			.background(ThemeColors.Dark.background),
	) {
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.Center,
			modifier = Modifier.fillMaxSize(),
		) {
			val logoAnimationState =
				animateLottieCompositionAsState(
					composition = composition.value,
					speed = 2.5f,
					iterations = 1,
					isPlaying = true,
					restartOnPlay = false,
					cancellationBehavior = LottieCancellationBehavior.Immediately,
				)

			LottieAnimation(
				composition = composition.value,
				progress = { logoAnimationState.progress },
			)
		}
	}
}
