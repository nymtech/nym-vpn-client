package net.nymtech.nymvpn.ui.screens.technical.components

import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun WelcomeSection(modifier: Modifier = Modifier) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.CenterVertically),
		modifier = modifier,
	) {
		Image(
			painter = painterResource(R.drawable.app_label),
			contentDescription = stringResource(R.string.app_name),
			colorFilter = ColorFilter.tint(MaterialTheme.colorScheme.onPrimaryContainer),
			contentScale = ContentScale.Fit,
			modifier = Modifier.size(110.dp.scaledWidth()),
			alignment = Alignment.Center,
		)
		Text(
			text = stringResource(R.string.welcome_title),
			style = MaterialTheme.typography.titleLarge,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
		Text(
			text = stringResource(R.string.welcome_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onBackground,
			textAlign = TextAlign.Center,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
internal fun PreviewWelcomeSection() {
	NymVPNTheme(Theme.default()) {
		WelcomeSection()
	}
}
