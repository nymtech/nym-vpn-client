import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextLinkStyles
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withLink
import net.nymtech.nymvpn.R

@Composable
fun PrivacyText() {
	val uriHandler = LocalUriHandler.current
	val termsUrl = stringResource(R.string.terms_link)
	val privacyUrl = stringResource(R.string.privacy_link)

	Text(
		text = buildAnnotatedString {
			append(stringResource(R.string.account_welcome_privacy_start))
			append("\n")

			withLink(
				LinkAnnotation.Clickable(
					tag = "terms",
					styles = TextLinkStyles(
						style = SpanStyle(
							color = MaterialTheme.colorScheme.onSurface,
							textDecoration = TextDecoration.None,
							fontWeight = FontWeight.ExtraBold,
						),
					),
					linkInteractionListener = { uriHandler.openUri(termsUrl) },
				),
			) { append(stringResource(R.string.terms_of_use)) }

			append(" ")
			append(stringResource(R.string.account_welcome_privacy_middle))
			append(" ")

			withLink(
				LinkAnnotation.Clickable(
					tag = "privacy",
					styles = TextLinkStyles(
						style = SpanStyle(
							color = MaterialTheme.colorScheme.onSurface,
							textDecoration = TextDecoration.None,
							fontWeight = FontWeight.ExtraBold,
						),
					),
					linkInteractionListener = { uriHandler.openUri(privacyUrl) },
				),
			) { append(stringResource(R.string.privacy_policy)) }

			append(".")
		},
		textAlign = TextAlign.Center,
		style = MaterialTheme.typography.bodyMedium.copy(
			color = MaterialTheme.colorScheme.onSurface,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		),
		modifier = Modifier.fillMaxWidth(),
	)
}
