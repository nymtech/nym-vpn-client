package net.nymtech.nymvpn.ui.screens.onboarding

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerState
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectMode
import net.nymtech.nymvpn.ui.screens.onboarding.components.OnboardingBottomCard
import net.nymtech.nymvpn.ui.screens.onboarding.components.OnboardingPage
import net.nymtech.nymvpn.ui.screens.onboarding.components.OnboardingPageContent
import net.nymtech.nymvpn.ui.screens.onboarding.components.OnboardingPlanPricing
import net.nymtech.nymvpn.ui.screens.onboarding.components.PagerIndicator
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

private val DEFAULT_PAGES = OnboardingPage.entries.filterNot { it == OnboardingPage.PLAN }

@Composable
fun OnboardingScreen(viewModel: OnboardingViewModel = hiltViewModel()) {
	val navigator = LocalNavController.current
	val scope = rememberCoroutineScope()
	val planPricing by viewModel.planPricing.collectAsStateWithLifecycle()
	val pages = if (viewModel.isPlanPageEnabled) OnboardingPage.entries else DEFAULT_PAGES

	OnboardingScreen(
		pages = pages,
		planPricing = planPricing,
		onFinish = {
			scope.launch {
				viewModel.onOnboardingCompleted()
				navigator.navigateAndForget(Route.Main())
			}
		},
	)
}

@Composable
fun OnboardingScreen(pages: List<OnboardingPage> = DEFAULT_PAGES, planPricing: OnboardingPlanPricing? = null, onFinish: () -> Unit) {
	val scope = rememberCoroutineScope()
	var selectedMode by remember { mutableStateOf(ConnectMode.FAST) }

	val pagerState = rememberPagerState(
		initialPage = 0,
		pageCount = { pages.size },
	)

	val tagline = if (pages[pagerState.currentPage] == OnboardingPage.MODES) {
		stringResource(
			if (selectedMode == ConnectMode.MIXNET) R.string.onboarding_modes_tagline_mixnet else R.string.onboarding_modes_tagline_dvpn,
		)
	} else {
		null
	}

	Box(
		modifier = Modifier
			.fillMaxSize()
			.background(MaterialTheme.colorScheme.background),
	) {
		if (pages[pagerState.currentPage] == OnboardingPage.PLAN) {
			PlanGlowBackground()
		}

		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier
				.fillMaxSize()
				.padding(horizontal = 16.dp.scaledWidth())
				.padding(WindowInsets.systemBars.asPaddingValues()),
		) {
			HorizontalPager(
				state = pagerState,
				modifier = Modifier
					.fillMaxWidth()
					.weight(1f),
			) { page ->
				OnboardingPageContent(
					page = pages[page],
					selectedMode = selectedMode,
					onSelectedModeChange = { selectedMode = it },
					pricing = planPricing,
					modifier = Modifier.fillMaxWidth(),
				)
			}

			OnboardingNavRow(pagerState = pagerState, pageCount = pages.size, scope = scope)

			Spacer(modifier = Modifier.height(18.dp.scaledHeight()))

			OnboardingBottomCard(
				onGetStartedClick = onFinish,
				modifier = Modifier.padding(bottom = 24.dp.scaledHeight()),
				tagline = tagline,
			)
		}
	}
}

@Composable
private fun OnboardingNavRow(pagerState: PagerState, pageCount: Int, scope: CoroutineScope, modifier: Modifier = Modifier) {
	val lastIndex = pageCount - 1

	Row(
		verticalAlignment = Alignment.CenterVertically,
		modifier = modifier.fillMaxWidth(),
	) {
		if (pagerState.currentPage > 0) {
			Icon(
				imageVector = Icons.AutoMirrored.Filled.KeyboardArrowLeft,
				contentDescription = stringResource(R.string.previous),
				tint = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier
					.size(40.dp)
					.clip(CircleShape)
					.background(color = MaterialTheme.colorScheme.surfaceContainer)
					.clickable {
						scope.launch { pagerState.animateScrollToPage(pagerState.currentPage - 1) }
					}
					.padding(10.dp),
			)
		} else {
			Spacer(modifier = Modifier.size(40.dp))
		}

		Spacer(modifier = Modifier.weight(1f))

		Row(
			modifier = Modifier
				.background(
					color = MaterialTheme.colorScheme.surfaceContainer,
					shape = RoundedCornerShape(50),
				)
				.padding(horizontal = 12.dp, vertical = 8.dp),
		) {
			PagerIndicator(
				pageCount = pageCount,
				currentPage = pagerState.currentPage,
			)
		}

		Spacer(modifier = Modifier.weight(1f))

		if (pagerState.currentPage < lastIndex) {
			Icon(
				imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
				contentDescription = stringResource(R.string.next),
				tint = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier
					.size(40.dp)
					.clip(CircleShape)
					.background(color = MaterialTheme.colorScheme.surfaceContainer)
					.clickable {
						scope.launch { pagerState.animateScrollToPage(pagerState.currentPage + 1) }
					}
					.padding(10.dp),
			)
		} else {
			Spacer(modifier = Modifier.size(40.dp))
		}
	}
}

@Composable
private fun BoxScope.PlanGlowBackground() {
	val glowColor = MaterialTheme.colorScheme.primary

	GlowCircle(
		color = glowColor,
		size = 260.dp,
		modifier = Modifier.align(Alignment.CenterStart).offset(x = (-130).dp),
	)

	GlowCircle(
		color = glowColor,
		size = 170.dp,
		modifier = Modifier.align(Alignment.TopEnd).offset(x = 40.dp, y = 180.dp),
	)
	GlowCircle(
		color = glowColor,
		size = 180.dp,
		modifier = Modifier.align(Alignment.TopEnd).offset(x = 30.dp, y = 310.dp),
	)

	GlowCircle(
		color = glowColor,
		size = 200.dp,
		modifier = Modifier.align(Alignment.BottomEnd).offset(x = 20.dp, y = (-160).dp),
	)
}

@Composable
private fun GlowCircle(color: Color, size: Dp, modifier: Modifier = Modifier) {
	val brush = remember(color) {
		val steps = 16
		Brush.radialGradient(
			colorStops = Array(steps + 1) { i ->
				val t = i / steps.toFloat()
				val alpha = GLOW_PEAK_ALPHA * (1f - t) * (1f - t)
				t to color.copy(alpha = alpha)
			},
		)
	}
	Box(modifier = modifier.size(size).background(brush))
}

private const val GLOW_PEAK_ALPHA = 0.5f

@Composable
@PreviewLightDark
internal fun PreviewOnboardingScreen() {
	NymVPNTheme(Theme.default()) {
		OnboardingScreen(onFinish = {})
	}
}

@Composable
@PreviewLightDark
internal fun PreviewOnboardingPlanSlide() {
	NymVPNTheme(Theme.default()) {
		OnboardingScreen(
			pages = listOf(OnboardingPage.PLAN),
			planPricing = OnboardingPlanPricing(
				monthlyEquivalentPrice = "$8.26",
				savingsPercent = "65%",
				freeTrialDays = 7,
			),
			onFinish = {},
		)
	}
}
