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
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.openWebUrl

@Composable
fun DetailsSectionIP(exitIpv4: String?, exitIpv6: String?, asn: String?, asnName: String?) {
	val context = LocalContext.current
	val interactionSource = remember { MutableInteractionSource() }

	val items = buildList<Pair<String, @Composable () -> Unit>> {
		exitIpv4?.let { ipv4 ->
			val url = stringResource(R.string.details_ip_link, ipv4)
			add(
				stringResource(R.string.details_exit_ipv4) to {
					Row(
						verticalAlignment = Alignment.CenterVertically,
						modifier = Modifier
							.clickable(interactionSource = interactionSource, indication = null) {
								context.openWebUrl(url)
							},
					) {
						Text(
							text = ipv4,
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
				},
			)
		}

		exitIpv6?.let { ipv6 ->
			val url = stringResource(R.string.details_ip_link, ipv6)
			add(
				stringResource(R.string.details_exit_ipv6) to {
					Row(
						verticalAlignment = Alignment.CenterVertically,
						modifier = Modifier
							.clickable(interactionSource = interactionSource, indication = null) {
								context.openWebUrl(url)
							},
					) {
						Text(
							text = ipv6,
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
				},
			)
		}

		asn?.let {
			add(
				stringResource(R.string.details_asn) to {
					Text(
						text = it,
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
				},
			)
		}

		asnName?.let {
			add(
				stringResource(R.string.details_asn_name) to {
					Text(
						text = it,
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
				},
			)
		}
	}

	InfoSection(
		titleResId = R.string.details_features_title, items = items)
}

@Composable
@PreviewLightDark
private fun PreviewDetailsSectionIP() {
	NymVPNTheme(Theme.default()) {
		Surface {
			DetailsSectionIP(
				exitIpv4 = "12.34.152.125",
				exitIpv6 = "12:ff:14::155",
				asn = "AS29234",
				asnName = "Google LLC",
			)
		}
	}
}
