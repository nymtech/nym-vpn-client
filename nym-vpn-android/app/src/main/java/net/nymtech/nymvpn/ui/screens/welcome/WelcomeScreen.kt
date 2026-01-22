package net.nymtech.nymvpn.ui.screens.welcome

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.OutlineStyledButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.welcome.components.OnboardingPage
import net.nymtech.nymvpn.ui.screens.welcome.components.OnboardingPageContent
import net.nymtech.nymvpn.ui.screens.welcome.components.PagerIndicator
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun WelcomeScreen(viewModel: WelcomeViewModel = hiltViewModel()) {
	val navigator = LocalNavController.current
	WelcomeScreen(
		withFreeTrial = true,
		onCreateAccountClick = {
			navigator.goFromRoot(Route.CreateAccount)
		},
		onLoginClick = {
			navigator.goFromRoot(Route.Login)
		},
		onCloseClick = {
			navigator.goFromRoot(Route.Main())
		},
	)
}

@Composable
fun WelcomeScreen(withFreeTrial: Boolean, onCreateAccountClick: () -> Unit, onLoginClick: () -> Unit, onCloseClick: () -> Unit) {
	val scope = rememberCoroutineScope()
	val pages = getPages(withFreeTrial)
	val lastIndex = pages.lastIndex

	val pagerState = rememberPagerState(
		initialPage = 0,
		pageCount = { pages.size },
	)

	LaunchedEffect(pagerState, lastIndex) {
		snapshotFlow { pagerState.currentPage to pagerState.currentPageOffsetFraction }
			.collect { (page, offset) ->
				if (page == 0 && offset < -0.5f) {
					pagerState.scrollToPage(lastIndex)
				}
				if (page == lastIndex && offset > 0.5f) {
					pagerState.scrollToPage(0)
				}
			}
	}

	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = Modifier
			.fillMaxSize()
			.imePadding()
			.padding(horizontal = 24.dp.scaledWidth())
			.padding(WindowInsets.systemBars.asPaddingValues()),
	) {
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.Top,
			modifier = Modifier
				.fillMaxWidth()
				.weight(1f),
		) {
			HorizontalPager(
				state = pagerState,
				modifier = Modifier
					.fillMaxWidth()
					.weight(1f, fill = false),
			) { page ->
				OnboardingPageContent(
					title = pages[page].title,
					description = pages[page].description,
					image = pages[page].image,
					modifier = Modifier.fillMaxWidth(),
				)
			}

			Spacer(modifier = Modifier.height(16.dp.scaledHeight()))

			Row(
				verticalAlignment = Alignment.CenterVertically,
				modifier = Modifier.fillMaxWidth(),
			) {
				Icon(
					imageVector = Icons.AutoMirrored.Filled.KeyboardArrowLeft,
					contentDescription = stringResource(R.string.app_name),
					tint = MaterialTheme.colorScheme.onBackground,
					modifier = Modifier
						.size(40.dp)
						.background(
							color = MaterialTheme.colorScheme.surfaceContainer,
							shape = CircleShape,
						)
						.clickable {
							val prev =
								if (pagerState.currentPage == 0) {
									lastIndex
								} else {
									pagerState.currentPage - 1
								}
							scope.launch { pagerState.animateScrollToPage(prev) }
						}
						.padding(10.dp),
				)

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
						pageCount = pages.size,
						currentPage = pagerState.currentPage,
					)
				}

				Spacer(modifier = Modifier.weight(1f))

				Icon(
					imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
					contentDescription = stringResource(R.string.app_name),
					tint = MaterialTheme.colorScheme.onBackground,
					modifier = Modifier
						.size(40.dp)
						.background(
							color = MaterialTheme.colorScheme.surfaceContainer,
							shape = CircleShape,
						)
						.clickable {
							val next =
								if (pagerState.currentPage == lastIndex) {
									0
								} else {
									pagerState.currentPage + 1
								}
							scope.launch { pagerState.animateScrollToPage(next) }
						}
						.padding(10.dp),
				)
			}
		}

		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier
				.fillMaxWidth()
				.padding(bottom = 24.dp.scaledHeight()),
		) {
			MainStyledButton(
				onClick = onCreateAccountClick,
				content = {
					Text(
						text = stringResource(R.string.onboarding_create_account_button),
						style = CustomTypography.buttonMain,
					)
				},
				color = MaterialTheme.colorScheme.primary,
				modifier = Modifier
					.fillMaxWidth()
					.height(54.dp.scaledHeight()),
			)

			Spacer(modifier = Modifier.height(24.dp.scaledHeight()))

			OutlineStyledButton(
				onClick = onLoginClick,
				content = {
					Text(
						text = stringResource(R.string.log_in),
						style = CustomTypography.buttonMain,
						color = MaterialTheme.colorScheme.onBackground,
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(52.dp.scaledHeight()),
				borderColor = MaterialTheme.colorScheme.onBackground,
				backgroundColor = MaterialTheme.colorScheme.background,
			)
		}
	}
}

@Composable
private fun getPages(withFreeTrial: Boolean): List<OnboardingPage> {
	return listOf(
		if (withFreeTrial) {
			OnboardingPage(
				title = stringResource(R.string.welcome_to_nym),
				description = stringResource(R.string.onboarding_description_1),
				image = R.drawable.img_onboarding_1,
			)
		} else {
			OnboardingPage(
				title = stringResource(R.string.welcome_to_nym),
				description = stringResource(R.string.onboarding_description_base),
				image = R.drawable.img_onboarding_base,
			)
		},
		OnboardingPage(
			title = stringResource(R.string.onboarding_title_2),
			description = stringResource(R.string.onboarding_description_2),
			image = R.drawable.img_onboarding_2,
		),
		OnboardingPage(
			title = stringResource(R.string.onboarding_title_3),
			description = stringResource(R.string.onboarding_description_3),
			image = R.drawable.img_onboarding_3,
		),
		OnboardingPage(
			title = stringResource(R.string.onboarding_title_4),
			description = stringResource(R.string.onboarding_description_4),
			image = R.drawable.img_onboarding_4,
		),
	)
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
internal fun PreviewWelcomeScreen() {
	NymVPNTheme(Theme.default()) {
		WelcomeScreen(
			withFreeTrial = true,
			onCreateAccountClick = {},
			onLoginClick = {},
			onCloseClick = {},
		)
	}
}
