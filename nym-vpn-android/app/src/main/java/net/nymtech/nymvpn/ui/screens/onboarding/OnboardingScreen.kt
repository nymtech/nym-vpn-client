package net.nymtech.nymvpn.ui.screens.onboarding

import android.content.res.Configuration
import androidx.compose.foundation.Image
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
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.onboarding.components.OnboardingPage
import net.nymtech.nymvpn.ui.screens.onboarding.components.OnboardingPageContent
import net.nymtech.nymvpn.ui.screens.onboarding.components.PagerIndicator
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun OnboardingScreen(viewModel: OnboardingViewModel = hiltViewModel()) {
	val navigator = LocalNavController.current
	OnboardingScreen(
		onContinueClick = {
			viewModel.onContinueClicked()
			navigator.goFromRoot(Route.WelcomeAccount(true))
		},
	)
}

@Composable
fun OnboardingScreen(onContinueClick: () -> Unit) {
	val scope = rememberCoroutineScope()
	val pages = getPages()

	val pagerState = rememberPagerState(
		initialPage = 0,
		pageCount = { pages.size },
	)

	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = Modifier
			.fillMaxSize()
			.imePadding()
			.padding(horizontal = 24.dp.scaledWidth())
			.padding(WindowInsets.systemBars.asPaddingValues()),
	) {
		Row(
			verticalAlignment = Alignment.CenterVertically,
			horizontalArrangement = Arrangement.End,
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 24.dp.scaledHeight()),
		) {
			Text(
				text = stringResource(R.string.skip),
				style = MaterialTheme.typography.titleMedium,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				color = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.clickable { onContinueClick() },
			)
		}

		Spacer(modifier = Modifier.height(16.dp.scaledHeight()))

		Image(
			painter = painterResource(R.drawable.app_label),
			contentDescription = stringResource(R.string.app_name),
			colorFilter = ColorFilter.tint(MaterialTheme.colorScheme.onBackground),
			contentScale = ContentScale.Fit,
			modifier = Modifier.size(110.dp.scaledWidth()),
			alignment = Alignment.Center,
		)

		Spacer(modifier = Modifier.height(8.dp.scaledHeight()))

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

			Spacer(modifier = Modifier.height(24.dp.scaledHeight()))

			PagerIndicator(
				pageCount = pages.size,
				currentPage = pagerState.currentPage,
				modifier = Modifier.padding(bottom = 12.dp.scaledHeight()),
			)
		}

		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier
				.fillMaxWidth()
				.padding(bottom = 32.dp.scaledHeight()),
		) {
			val lastIndex = pages.lastIndex
			MainStyledButton(
				onClick = {
					if (pagerState.currentPage < lastIndex) {
						scope.launch { pagerState.animateScrollToPage(pagerState.currentPage + 1) }
					} else {
						onContinueClick()
					}
				},
				content = {
					val textId = if (pagerState.currentPage == lastIndex) R.string.welcome_continue else R.string.button_next
					Text(
						text = stringResource(textId),
						style = CustomTypography.buttonMain,
					)
				},
				color = MaterialTheme.colorScheme.primary,
				modifier = Modifier
					.fillMaxWidth()
					.height(54.dp.scaledHeight()),
			)
		}
	}
}

@Composable
private fun getPages(): List<OnboardingPage> {
	return listOf(
		OnboardingPage(
			title = stringResource(R.string.welcome_to_nym),
			description = stringResource(R.string.onboarding_description_1),
			image = R.drawable.img_onboarding_1,
		),
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
internal fun PreviewOnboardingScreen() {
	NymVPNTheme(Theme.default()) {
		OnboardingScreen {}
	}
}
