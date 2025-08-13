package net.nymtech.nymvpn.ui.screens.welcome.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R

@Composable
fun WelcomeSection(modifier: Modifier = Modifier) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.CenterVertically),
		modifier = modifier,
	) {
		Text(
			text = stringResource(R.string.welcome_to_nym).uppercase(),
			style = MaterialTheme.typography.headlineSmall,
			color = MaterialTheme.colorScheme.primary,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
		)
		Text(
			text = buildAnnotatedString {
				append(stringResource(R.string.welcome_descr_start))
				append(" ")
				withStyle(
					style = SpanStyle(
						color = MaterialTheme.colorScheme.primary,
						textDecoration = TextDecoration.Underline,
					),
				) {
					withLink(LinkAnnotation.Url(stringResource(R.string.welcome_descr_sentry_link))) {
						append(stringResource(R.string.welcome_descr_sentry_link_text))
					}
				}
				append(stringResource(R.string.welcome_descr_end))
			},
			style = MaterialTheme.typography.bodyLarge,
			textAlign = TextAlign.Center,
		)
	}
}
