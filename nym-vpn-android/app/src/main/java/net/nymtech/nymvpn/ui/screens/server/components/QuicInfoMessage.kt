package net.nymtech.nymvpn.ui.screens.server.components

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.Typography

@Composable
internal fun QuicInfoMessage(onNavigateToQuicSettings: () -> Unit) {
	val annotatedText = buildAnnotatedString {
		append(stringResource(R.string.quic_gateway_filter_info_msg))
		append(" ")
		withStyle(
			style = SpanStyle(
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				textDecoration = TextDecoration.Underline,
			),
		) {
			withLink(
				LinkAnnotation.Clickable("quic", linkInteractionListener = { onNavigateToQuicSettings() }),
			) { append(stringResource(R.string.here)) }
		}
		append(".")
	}

	Text(
		text = annotatedText,
		style = Typography.bodyMedium.copy(
			color = MaterialTheme.colorScheme.onBackground,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		),
	)
}
