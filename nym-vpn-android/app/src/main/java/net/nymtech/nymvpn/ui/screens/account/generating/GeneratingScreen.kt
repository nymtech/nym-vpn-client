package net.nymtech.nymvpn.ui.screens.account.generating

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.delay
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.AuthRoute
import net.nymtech.nymvpn.ui.routeName
import net.nymtech.nymvpn.ui.common.animations.PulsingDotsWave
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.AlertController
import net.nymtech.nymvpn.ui.common.snackbar.AlertMessage
import net.nymtech.nymvpn.ui.common.snackbar.AlertType
import net.nymtech.nymvpn.ui.theme.*
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.navigateAndForgetToMain
import kotlin.time.Duration.Companion.milliseconds

@Composable
fun GeneratingScreen(viewModel: GeneratingViewModel = hiltViewModel()) {
	val readyForSelectPlan by viewModel.readyForSelectPlan.collectAsStateWithLifecycle()
	val navController = LocalNavController.current
	val mode = viewModel.mode
	val errorText = stringResource(R.string.account_generating_error)
	var animationEnded by remember { mutableStateOf(false) }

	LaunchedEffect(Unit) {
		viewModel.error.collect {
			AlertController.show(AlertMessage(type = AlertType.Error, title = errorText))
			navController.navigateAndForget(Route.Main(authRoute = AuthRoute.Welcome.routeName))
		}
	}

	LaunchedEffect(animationEnded, readyForSelectPlan) {
		if (!readyForSelectPlan || !animationEnded) return@LaunchedEffect
		navController.navigateAndForgetToMain(Route.SelectPlan)
	}

	GeneratingContent(
		mode = mode,
		onAnimationEnd = {
			if (mode == GeneratingMode.CreateAccount) {
				animationEnded = true
			}
		},
	)
}

@Composable
fun GeneratingContent(mode: GeneratingMode, onAnimationEnd: () -> Unit) {
	var step by remember { mutableIntStateOf(0) }
	val isDeepLink = mode == GeneratingMode.DeepLinkLogin

	val creationSteps = remember {
		listOf(
			R.string.account_generating_title_1 to R.string.account_generating_description_1,
			R.string.account_generating_title_2 to R.string.account_generating_description_2,
			R.string.account_generating_title_3 to R.string.account_generating_description_3,
		)
	}

	LaunchedEffect(isDeepLink) {
		if (!isDeepLink) {
			repeat(2) {
				delay(3000.milliseconds)
				step++
			}
			delay(3000.milliseconds)
			onAnimationEnd()
		}
	}

	val resPair = if (isDeepLink) {
		R.string.account_generating_deeplink_title to R.string.account_generating_deeplink_description
	} else {
		creationSteps[step.coerceIn(0, 2)]
	}

	GeneratingBaseLayout(
		title = stringResource(resPair.first),
		description = stringResource(resPair.second),
		topContent = if (!isDeepLink) {
			{ StepProgressBar(step) }
		} else {
			null
		},
	)
}

@Composable
private fun GeneratingBaseLayout(title: String, description: String, topContent: @Composable (() -> Unit)?) {
	Column(
		modifier = Modifier
			.fillMaxSize()
			.background(MaterialTheme.colorScheme.background),
		horizontalAlignment = Alignment.CenterHorizontally,
	) {
		topContent?.invoke()
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier.padding(top = 200.dp),
		) {
			val nymColors = LocalNymColors.current
			Box(
				modifier = Modifier
					.size(56.dp)
					.background(nymColors.iconBackground, RoundedCornerShape(8.dp))
					.border(1.dp, nymColors.iconBorder, RoundedCornerShape(8.dp)),
				contentAlignment = Alignment.Center,
			) {
				PulsingDotsWave(modifier = Modifier.padding(8.dp))
			}
			listOf(
				title to (MaterialTheme.typography.titleMedium to 24.dp),
				description to (MaterialTheme.typography.bodyMedium to 8.dp),
			).forEach { (text, stylePair) ->
				Text(
					text = text,
					style = stylePair.first,
					color = if (stylePair.first == MaterialTheme.typography.titleMedium) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onBackground,
					textAlign = TextAlign.Center,
					modifier = Modifier.padding(top = stylePair.second, start = 32.dp, end = 32.dp),
				)
			}
		}
	}
}

@Composable
private fun StepProgressBar(step: Int) {
	Row(
		modifier = Modifier
			.padding(horizontal = 16.dp)
			.fillMaxWidth()
			.height(4.dp),
		horizontalArrangement = Arrangement.spacedBy(4.dp),
	) {
		repeat(4) { i ->
			val active = i < 3 && step >= i
			Box(
				modifier = Modifier
					.weight(1f)
					.fillMaxHeight()
					.background(
						if (active) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceContainer,
						RoundedCornerShape(4.dp),
					),
			)
		}
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, showBackground = true)
@Composable
private fun PreviewGeneratingScreens() {
	NymVPNTheme(Theme.default()) {
		Column { GeneratingContent(GeneratingMode.CreateAccount) {} }
	}
}

enum class GeneratingMode { CreateAccount, DeepLinkLogin }
