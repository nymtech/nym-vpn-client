package net.nymtech.nymvpn.ui.screens.welcome.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun ContinueButtonSection(onContinueClick: () -> Unit) {
	MainStyledButton(
		onClick = {
			onContinueClick()
		},
		content = {
			Text(
				stringResource(R.string.welcome_continue),
				style = CustomTypography.buttonMain,
			)
		},
		color = MaterialTheme.colorScheme.primary,
		modifier = Modifier
			.fillMaxWidth()
			.height(56.dp.scaledHeight()),
	)
	Text(
		text = buildAnnotatedString {
			append(stringResource(R.string.welcome_terms_privacy_text_start))
			append(" ")
			withStyle(
				style = SpanStyle(
					color = MaterialTheme.colorScheme.primary,
					textDecoration = TextDecoration.Underline,
				),
			) {
				withLink(LinkAnnotation.Url(stringResource(R.string.terms_link))) {
					append(stringResource(R.string.terms_of_use))
				}
			}
			append(" ")
			append(stringResource(R.string.welcome_terms_privacy_text_middle))
			append(" ")
			withStyle(
				style = SpanStyle(
					color = MaterialTheme.colorScheme.primary,
					textDecoration = TextDecoration.Underline,
				),
			) {
				withLink(LinkAnnotation.Url(stringResource(R.string.privacy_link))) {
					append(stringResource(R.string.privacy_policy))
				}
			}
		},
		style = MaterialTheme.typography.labelSmall,
		textAlign = TextAlign.Center,
	)
}
