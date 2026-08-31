package net.nymtech.nymvpn.ui.screens.main.bottomsheet.processing

import android.content.res.Configuration
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import kotlinx.coroutines.delay
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.animations.PulsingDotsWave
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import nym_vpn_lib_types.AccountControllerState
import kotlin.time.Duration.Companion.milliseconds

@Composable
fun ProcessingComponent(onProcessingComplete: () -> Unit, authSheetMinHeightPx: Int = 0, viewModel: LoginProcessingViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val uiPhase by viewModel.uiPhase.collectAsState()
	val progressStep by viewModel.progressStep.collectAsState()
	val navigationRoute by viewModel.navigationRoute.collectAsState()
	val timedOut by viewModel.timedOut.collectAsState()
	val failureMessageRes by viewModel.failureMessageRes.collectAsState()
	val accountState by viewModel.accountState.collectAsState()
	val credentialsCarouselTick by viewModel.credentialsCarouselTick.collectAsState()
	val setupCarouselIndex by viewModel.setupCarouselIndex.collectAsState()
	val setupCarouselFinished by viewModel.setupCarouselFinished.collectAsState()

	LaunchedEffect(Unit) {
		viewModel.startProcessing()
	}

	LaunchedEffect(navigationRoute, timedOut, failureMessageRes) {
		val destination = navigationRoute ?: return@LaunchedEffect
		failureMessageRes?.let { messageRes ->
			SnackbarController.showMessage(StringValue.StringResource(messageRes))
			viewModel.consumeFailureMessageRes()
		}
		if (timedOut) {
			SnackbarController.showMessage(StringValue.StringResource(R.string.account_generating_error))
		}
		viewModel.consumeNavigationRoute()
		onProcessingComplete()
		navController.navigateAndForget(destination)
	}

	ProcessingComponentContent(
		uiPhase = uiPhase,
		progressStep = progressStep,
		accountState = accountState,
		credentialsCarouselTick = credentialsCarouselTick,
		setupCarouselIndex = setupCarouselIndex,
		setupCarouselFinished = setupCarouselFinished,
		authSheetMinHeightPx = authSheetMinHeightPx,
	)
}

@Composable
fun ProcessingComponentContent(
	uiPhase: LoginProcessingUiPhase,
	progressStep: Int,
	accountState: AccountControllerState? = null,
	credentialsCarouselTick: Int = 0,
	setupCarouselIndex: Int = 0,
	setupCarouselFinished: Boolean = false,
	authSheetMinHeightPx: Int = 0,
) {
	val density = LocalDensity.current
	val minHeight = if (authSheetMinHeightPx > 0) {
		with(density) { authSheetMinHeightPx.toDp() }
	} else {
		ProcessingCopy.LOGIN_PROCESSING_MIN_HEIGHT_DP.dp
	}
	val processingCopy = ProcessingCopy.processingCopyForPhase(
		uiPhase,
		accountState,
		credentialsCarouselTick,
		setupCarouselIndex,
		setupCarouselFinished,
	)

	Column(
		modifier = Modifier
			.fillMaxWidth()
			.height(minHeight)
			.padding(horizontal = ProcessingCopy.LOGIN_PROCESSING_HORIZONTAL_PADDING_DP.dp)
			.padding(
				top = ProcessingCopy.LOGIN_PROCESSING_TOP_PADDING_DP.dp,
				bottom = ProcessingCopy.LOGIN_PROCESSING_BOTTOM_PADDING_DP.dp,
			),
		horizontalAlignment = Alignment.CenterHorizontally,
	) {
		Icon(
			imageVector = ImageVector.vectorResource(R.drawable.app_label),
			contentDescription = stringResource(R.string.app_name),
			tint = MaterialTheme.colorScheme.onPrimaryContainer,
		)

		Spacer(modifier = Modifier.height(ProcessingCopy.LOGIN_PROCESSING_LOGO_STEP_SPACING_DP.dp))

		LoginProcessingStepBar(targetStep = progressStep)

		Spacer(modifier = Modifier.weight(1f))

		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier.fillMaxWidth(),
		) {
			val nymColors = LocalNymColors.current
			Box(
				modifier = Modifier
					.size(56.dp)
					.background(
						color = nymColors.iconBackground,
						shape = RoundedCornerShape(size = 8.dp),
					)
					.border(
						width = 1.dp,
						color = nymColors.iconBorder,
						shape = RoundedCornerShape(size = 8.dp),
					),
			) {
				PulsingDotsWave(
					modifier = Modifier
						.align(Alignment.Center)
						.padding(8.dp),
				)
			}

			AnimatedContent(
				targetState = processingCopy,
				transitionSpec = {
					fadeIn(tween(ProcessingCopy.STEP_BAR_FILL_MS + 100, easing = LinearEasing)) togetherWith
						fadeOut(tween(ProcessingCopy.STEP_BAR_FILL_MS, easing = LinearEasing))
				},
				label = "loginProcessingCopy",
			) { animatedCopy ->
				Column(
					horizontalAlignment = Alignment.CenterHorizontally,
					modifier = Modifier
						.fillMaxWidth()
						.padding(top = 16.dp, start = 24.dp, end = 24.dp),
				) {
					Text(
						text = stringResource(animatedCopy.titleRes),
						style = MaterialTheme.typography.titleMedium,
						textAlign = TextAlign.Center,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
						modifier = Modifier.fillMaxWidth(),
					)
					animatedCopy.subtitleRes?.let { subtitleRes ->
						Text(
							text = stringResource(subtitleRes),
							style = MaterialTheme.typography.bodyMedium,
							textAlign = TextAlign.Center,
							color = MaterialTheme.colorScheme.onSurfaceVariant,
							modifier = Modifier
								.fillMaxWidth()
								.padding(top = 8.dp),
						)
					}
				}
			}
		}

		Spacer(modifier = Modifier.weight(1f))
	}
}

@Composable
private fun LoginProcessingStepBar(targetStep: Int) {
	var displayedStep by remember { mutableIntStateOf(0) }
	var initialFillDone by remember { mutableStateOf(false) }

	LaunchedEffect(Unit) {
		delay(ProcessingCopy.STEP_BAR_INITIAL_DELAY_MS)
		val initialTarget = targetStep.coerceIn(0, ProcessingCopy.LOGIN_PROGRESS_STEP_COUNT)
		if (initialTarget > 0) {
			for (step in 1..initialTarget.coerceAtMost(ProcessingCopy.LOGIN_INITIAL_PROGRESS_STEP)) {
				displayedStep = step
				delay(
					(
						ProcessingCopy.STEP_BAR_FILL_MS.toLong() +
							ProcessingCopy.STEP_BAR_INITIAL_PAUSE_MS
						).milliseconds,
				)
			}
		}
		initialFillDone = true
	}

	LaunchedEffect(targetStep, initialFillDone) {
		if (!initialFillDone || targetStep <= displayedStep) return@LaunchedEffect
		for (step in (displayedStep + 1)..targetStep.coerceAtMost(ProcessingCopy.LOGIN_PROGRESS_STEP_COUNT)) {
			displayedStep = step
			delay(ProcessingCopy.STEP_BAR_FORWARD_PAUSE_MS)
		}
	}

	Row(
		modifier = Modifier
			.fillMaxWidth()
			.height(4.dp),
		horizontalArrangement = Arrangement.spacedBy(4.dp),
	) {
		repeat(ProcessingCopy.LOGIN_PROGRESS_STEP_COUNT) { index ->
			LoginProcessingStepSegment(
				segmentStep = index + 1,
				displayedStep = displayedStep,
			)
		}
	}
}

@Composable
private fun RowScope.LoginProcessingStepSegment(segmentStep: Int, displayedStep: Int) {
	val targetFill = if (segmentStep <= displayedStep) 1f else 0f
	val fillScale by animateFloatAsState(
		targetValue = targetFill,
		animationSpec = tween(
			durationMillis = ProcessingCopy.STEP_BAR_FILL_MS,
			easing = LinearEasing,
		),
		label = "loginProcessingStepFill$segmentStep",
	)

	Box(
		modifier = Modifier
			.weight(1f)
			.fillMaxHeight()
			.background(
				color = MaterialTheme.colorScheme.surfaceContainer,
				shape = RoundedCornerShape(size = 4.dp),
			),
	) {
		Box(
			modifier = Modifier
				.matchParentSize()
				.graphicsLayer {
					scaleX = fillScale
					transformOrigin = TransformOrigin(0f, 0.5f)
				}
				.background(
					color = MaterialTheme.colorScheme.primary,
					shape = RoundedCornerShape(size = 4.dp),
				),
		)
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, showBackground = true)
@Composable
private fun PreviewProcessingComponentZkNyms() {
	NymVPNTheme(Theme.default()) {
		ProcessingComponentContent(
			uiPhase = LoginProcessingUiPhase.Carousel,
			progressStep = 4,
			accountState = AccountControllerState.Syncing,
			credentialsCarouselTick = 1,
		)
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, showBackground = true)
@Composable
private fun PreviewProcessingComponentCarousel() {
	NymVPNTheme(Theme.default()) {
		ProcessingComponentContent(
			uiPhase = LoginProcessingUiPhase.Carousel,
			progressStep = 2,
		)
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, showBackground = true)
@Composable
private fun PreviewProcessingComponentWelcome() {
	NymVPNTheme(Theme.default()) {
		ProcessingComponentContent(
			uiPhase = LoginProcessingUiPhase.Welcome,
			progressStep = 4,
		)
	}
}
