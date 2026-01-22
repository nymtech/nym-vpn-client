package net.nymtech.nymvpn.ui.screens.welcome.components

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun OnboardingPageContent(title: String, @DrawableRes image: Int, description: String, modifier: Modifier = Modifier) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(12.dp, Alignment.Top),
		modifier = modifier,
	) {
		Image(
			painter = painterResource(image),
			contentDescription = title,
			modifier = Modifier
				.height(300.dp.scaledHeight())
				.fillMaxWidth(),
			alignment = Alignment.Center,
		)
		Text(
			text = title,
			minLines = 2,
			style = MaterialTheme.typography.titleLarge,
			color = MaterialTheme.colorScheme.onBackground,
			textAlign = TextAlign.Center,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
		Text(
			text = description,
			minLines = 3,
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.outline,
			textAlign = TextAlign.Center,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
	}
}

data class OnboardingPage(
	@DrawableRes val image: Int,
	val title: String,
	val description: String,
)
