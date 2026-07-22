package net.nymtech.nymvpn.ui.screens.details.components

import android.widget.Toast
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography

@Composable
fun DetailsSectionIdentity(identity: String, buildVersion: String?) {
	val clipboardManager = LocalClipboardManager.current
	val context = LocalContext.current

	val items = buildList<Pair<String, @Composable () -> Unit>> {
		buildVersion?.let { version ->
			add(
				stringResource(R.string.details_build_version) to {
					Text(
						text = version,
						style = Typography.bodyMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				},
			)
		}
	}

	InfoSection(
		titleResId = R.string.details_features_title,
		items = items,
		bottomContent = {
			Row(
				verticalAlignment = Alignment.CenterVertically,
				modifier = Modifier
					.fillMaxWidth()
					.clickable {
						clipboardManager.setText(AnnotatedString(identity))
						Toast.makeText(context, "Copied to clipboard", Toast.LENGTH_SHORT).show()
					},
			) {
				Text(
					text = identity,
					style = Typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					maxLines = 2,
					overflow = TextOverflow.Ellipsis,
					modifier = Modifier.weight(1f),
				)
				Spacer(modifier = Modifier.width(8.dp))
				Icon(
					imageVector = Icons.Outlined.ContentCopy,
					contentDescription = null,
					tint = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier.size(16.dp),
				)
			}
		},
	)
}

@Composable
@PreviewLightDark
private fun PreviewDetailsSectionIdentity() {
	NymVPNTheme(Theme.default()) {
		Surface {
			DetailsSectionIdentity(identity = "wqewqewqewqewqfade2123123", buildVersion = "1.2.4")
		}
	}
}
