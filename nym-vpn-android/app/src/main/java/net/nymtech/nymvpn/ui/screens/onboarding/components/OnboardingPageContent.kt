package net.nymtech.nymvpn.ui.screens.onboarding.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import com.airbnb.lottie.LottieProperty
import com.airbnb.lottie.SimpleColorFilter
import com.airbnb.lottie.compose.LottieAnimation
import com.airbnb.lottie.compose.LottieCompositionSpec
import com.airbnb.lottie.compose.LottieConstants
import com.airbnb.lottie.compose.rememberLottieComposition
import com.airbnb.lottie.compose.rememberLottieDynamicProperties
import com.airbnb.lottie.compose.rememberLottieDynamicProperty
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectMode
import net.nymtech.nymvpn.ui.screens.main.panel.components.ModeTabs
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun OnboardingPageContent(page: OnboardingPage, selectedMode: ConnectMode, onSelectedModeChange: (ConnectMode) -> Unit, modifier: Modifier = Modifier, pricing: OnboardingPlanPricing? = null) {
	when (page) {
		OnboardingPage.WELCOME -> WelcomePage(modifier)
		OnboardingPage.MODES -> ModesPage(selectedMode, onSelectedModeChange, modifier)
		OnboardingPage.AROUND -> AroundPage(modifier)
		OnboardingPage.PLAN -> PlanPage(pricing, modifier)
	}
}

@Composable
private fun WelcomePage(modifier: Modifier = Modifier) {
	val composition by rememberLottieComposition(LottieCompositionSpec.RawRes(R.raw.noise_line))
	val secondaryColor = MaterialTheme.colorScheme.secondary
	val dynamicProperties = rememberLottieDynamicProperties(
		rememberLottieDynamicProperty(
			property = LottieProperty.COLOR_FILTER,
			value = SimpleColorFilter(secondaryColor.toArgb()),
			keyPath = arrayOf("**"),
		),
	)

	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(12.dp, Alignment.Top),
		modifier = modifier,
	) {
		LottieAnimation(
			composition = composition,
			iterations = LottieConstants.IterateForever,
			dynamicProperties = dynamicProperties,
			modifier = Modifier
				.height(200.dp.scaledHeight())
				.fillMaxWidth(),
		)
		OnboardingTitle(stringResource(R.string.onboarding_welcome_title))
		OnboardingDescription(boldMarkupText(stringResource(R.string.onboarding_welcome_description)))
	}
}

@Composable
private fun ModesPage(selectedMode: ConnectMode, onSelectedModeChange: (ConnectMode) -> Unit, modifier: Modifier = Modifier) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(12.dp, Alignment.Top),
		modifier = modifier,
	) {
		Image(
			painter = painterResource(
				if (selectedMode == ConnectMode.MIXNET) R.drawable.img_onboarding_2_2 else R.drawable.img_onboarding_2_1,
			),
			contentDescription = null,
			colorFilter = ColorFilter.tint(MaterialTheme.colorScheme.primary),
			modifier = Modifier
				.height(200.dp.scaledHeight())
				.fillMaxWidth(),
		)
		OnboardingTitle(
			stringResource(
				if (selectedMode == ConnectMode.MIXNET) R.string.onboarding_modes_title_mixnet else R.string.onboarding_modes_title_dvpn,
			),
		)
		OnboardingDescription(
			boldMarkupText(
				stringResource(
					if (selectedMode == ConnectMode.MIXNET) R.string.onboarding_modes_description_mixnet else R.string.onboarding_modes_description_dvpn,
				),
			),
		)
		ModeTabs(
			selected = selectedMode,
			onSelect = onSelectedModeChange,
			modifier = Modifier.padding(top = 8.dp.scaledHeight()),
		)
	}
}

@Composable
private fun AroundPage(modifier: Modifier = Modifier) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(12.dp, Alignment.Top),
		modifier = modifier,
	) {
		Image(
			painter = painterResource(R.drawable.img_onboarding_3),
			contentDescription = null,
			modifier = Modifier
				.height(200.dp.scaledHeight())
				.fillMaxWidth(),
		)
		OnboardingTitle(stringResource(R.string.onboarding_around_title))
		OnboardingDescription(boldMarkupText(stringResource(R.string.onboarding_around_description)))
	}
}

@Composable
private fun PlanPage(pricing: OnboardingPlanPricing?, modifier: Modifier = Modifier) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(12.dp, Alignment.Top),
		modifier = modifier,
	) {
		Box(
			contentAlignment = Alignment.Center,
			modifier = Modifier
				.fillMaxWidth(),
		) {
			Icon(
				imageVector = ImageVector.vectorResource(R.drawable.app_label),
				contentDescription = null,
				tint = MaterialTheme.colorScheme.onPrimaryContainer,
				modifier = Modifier
					.height(52.dp)
					.fillMaxWidth(0.5f),
			)
		}
		Spacer(modifier = Modifier.height(40.dp))
		OnboardingTitle(stringResource(R.string.onboarding_plan_title))
		OnboardingDescription(planSubtitle(pricing))
	}
}

@Composable
private fun planSubtitle(pricing: OnboardingPlanPricing?): AnnotatedString {
	val baseColor = MaterialTheme.colorScheme.onPrimaryContainer
	val highlightColor = MaterialTheme.colorScheme.primary
	val p = pricing ?: OnboardingPlanPricing.FALLBACK

	return buildAnnotatedString {
		fun base(text: String) = withStyle(SpanStyle(fontWeight = FontWeight.Bold, color = baseColor)) { append(text) }
		fun highlight(text: String) = withStyle(SpanStyle(fontWeight = FontWeight.Bold, color = highlightColor)) { append(text) }

		base(stringResource(R.string.onboarding_plan_community_favorite))
		append("\n\n")
		p.savingsPercent?.let { savings ->
			base(stringResource(R.string.onboarding_plan_year_plan_prefix))
			append(" ")
			highlight(stringResource(R.string.onboarding_plan_save_percent, savings))
			append("\n")
		}
		p.freeTrialDays?.let { days ->
			base(stringResource(R.string.onboarding_plan_free_trial, days))
			append("\n")
		}
		base(stringResource(R.string.onboarding_plan_starting_at_prefix))
		append(" ")
		highlight(stringResource(R.string.onboarding_plan_price_per_month, p.monthlyEquivalentPrice))
		append("\n\n")
		base(stringResource(R.string.onboarding_plan_try_prefix))
		base(stringResource(R.string.onboarding_plan_one_month))
		base(stringResource(R.string.onboarding_plan_try_suffix))
	}
}

private fun boldMarkupText(raw: String): AnnotatedString = buildAnnotatedString {
	raw.split("**").forEachIndexed { index, segment ->
		if (index % 2 == 1) {
			withStyle(SpanStyle(fontWeight = FontWeight.Bold)) { append(segment) }
		} else {
			append(segment)
		}
	}
}

@Composable
private fun OnboardingTitle(text: String) {
	Text(
		text = text,
		minLines = 2,
		style = MaterialTheme.typography.headlineSmall,
		color = MaterialTheme.colorScheme.onPrimaryContainer,
		textAlign = TextAlign.Center,
		fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
	)
}

@Composable
private fun OnboardingDescription(text: AnnotatedString) {
	Text(
		text = text,
		minLines = 3,
		style = MaterialTheme.typography.titleSmall,
		color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.9f),
		textAlign = TextAlign.Center,
		fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
	)
}
