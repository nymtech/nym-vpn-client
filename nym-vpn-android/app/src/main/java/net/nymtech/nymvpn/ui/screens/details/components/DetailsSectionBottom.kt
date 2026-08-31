package net.nymtech.nymvpn.ui.screens.details.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.openWebUrl

@Composable
fun DetailsSectionBottom(identity: String) {
	val context = LocalContext.current
	val interactionSource = remember { MutableInteractionSource() }
	Row(
		verticalAlignment = Alignment.CenterVertically,
		modifier = Modifier
			.clickable(
				interactionSource = interactionSource,
				indication = null,
			) {
				context.openWebUrl(context.getString(R.string.details_missing_incorrect_info_link))
			},
	) {
		Text(
			text = stringResource(R.string.details_missing_incorrect_info),
			style = MaterialTheme.typography.bodyMedium.copy(
				textDecoration = TextDecoration.Underline,
			),
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
		Spacer(modifier = Modifier.width(4.dp))
		Icon(
			imageVector = Icons.AutoMirrored.Outlined.OpenInNew,
			contentDescription = null,
			tint = MaterialTheme.colorScheme.onPrimaryContainer,
			modifier = Modifier.size(12.dp),
		)
	}

	Row(
		verticalAlignment = Alignment.CenterVertically,
		modifier = Modifier
			.clickable(
				interactionSource = interactionSource,
				indication = null,
			) {
				context.openWebUrl(context.getString(R.string.details_missing_incorrect_info_link))
			},
	) {
		val url = stringResource(R.string.details_more_details_link, identity)
		Text(
			text = buildAnnotatedString {
				withStyle(
					style = SpanStyle(
						color = MaterialTheme.colorScheme.onBackground,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					),
				) {
					append(stringResource(R.string.details_more_details_start))
				}
				append(" ")
				withStyle(
					style = SpanStyle(
						color = MaterialTheme.colorScheme.onPrimaryContainer,
						textDecoration = TextDecoration.Underline,
					),
				) {
					withLink(LinkAnnotation.Url(url = url)) {
						append(stringResource(R.string.details_more_details_end))
					}
				}
			},
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
		Spacer(modifier = Modifier.width(4.dp))
		Icon(
			imageVector = Icons.AutoMirrored.Outlined.OpenInNew,
			contentDescription = null,
			tint = MaterialTheme.colorScheme.onPrimaryContainer,
			modifier = Modifier.size(12.dp),
		)
	}
}

@Composable
@PreviewLightDark
private fun PreviewDetailsSectionBottom() {
	NymVPNTheme(Theme.default()) {
		Surface {
			DetailsSectionBottom(identity = "wqewqewqewqewqfade2123123")
		}
	}
}
