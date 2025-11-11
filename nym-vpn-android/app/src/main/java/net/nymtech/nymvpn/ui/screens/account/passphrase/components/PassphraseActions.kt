package net.nymtech.nymvpn.ui.screens.account.passphrase.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.OutlineStyledButton
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun PassphraseActions(show: Boolean, onCopyClick: () -> Unit, onDownloadClick: () -> Unit, onSaveClick: () -> Unit) {
	if (show) {
		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 16.dp),
			horizontalAlignment = Alignment.CenterHorizontally,
		) {
			OutlineStyledButton(
				modifier = Modifier
					.fillMaxWidth(0.8f)
					.padding(horizontal = 12.dp),
				onClick = onSaveClick,
				borderColor = MaterialTheme.colorScheme.outline,
				content = {
					Row(verticalAlignment = Alignment.CenterVertically) {
						Icon(
							imageVector = Icons.Outlined.Lock,
							contentDescription = null,
							tint = MaterialTheme.colorScheme.onSurface,
							modifier = Modifier.size(16.dp),
						)
						Spacer(modifier = Modifier.width(4.dp))
						Text(
							stringResource(R.string.passphrase_save),
							style = MaterialTheme.typography.bodyMedium,
							color = MaterialTheme.colorScheme.onSurface,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						)
					}
				},
			)
			Spacer(modifier = Modifier.height(16.dp))
			OutlineStyledButton(
				modifier = Modifier
					.fillMaxWidth(0.8f)
					.padding(horizontal = 12.dp),
				onClick = onDownloadClick,
				borderColor = MaterialTheme.colorScheme.outline,
				content = {
					Row(verticalAlignment = Alignment.CenterVertically) {
						Icon(
							imageVector = ImageVector.vectorResource(R.drawable.download),
							contentDescription = null,
							tint = MaterialTheme.colorScheme.onSurface,
							modifier = Modifier.size(16.dp),
						)
						Spacer(modifier = Modifier.width(4.dp))
						Text(
							stringResource(R.string.passphrase_download),
							style = MaterialTheme.typography.bodyMedium,
							color = MaterialTheme.colorScheme.onSurface,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						)
					}
				},
			)
			Spacer(modifier = Modifier.height(16.dp))
			OutlineStyledButton(
				modifier = Modifier
					.fillMaxWidth(0.8f)
					.padding(horizontal = 12.dp),
				onClick = onCopyClick,
				borderColor = MaterialTheme.colorScheme.outline,
				content = {
					Row(verticalAlignment = Alignment.CenterVertically) {
						Icon(
							imageVector = ImageVector.vectorResource(R.drawable.copy),
							contentDescription = null,
							tint = MaterialTheme.colorScheme.onSurface,
							modifier = Modifier.size(16.dp),
						)
						Spacer(modifier = Modifier.width(4.dp))
						Text(
							stringResource(R.string.passphrase_copy),
							style = MaterialTheme.typography.bodyMedium,
							color = MaterialTheme.colorScheme.onSurface,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						)
					}
				},
			)

			Text(
				text = stringResource(R.string.passphrase_lose),
				style = MaterialTheme.typography.labelLarge,
				color = MaterialTheme.colorScheme.onSurface,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				modifier = Modifier
					.fillMaxWidth()
					.padding(top = 24.dp),
				textAlign = TextAlign.Start,
			)
			Text(
				text = stringResource(R.string.passphrase_never),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.9f),
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				modifier = Modifier
					.fillMaxWidth()
					.padding(top = 4.dp),
				textAlign = TextAlign.Start,
			)
		}
	}
}

@Composable
@PreviewLightDark
internal fun PreviewPassphraseActions() {
	NymVPNTheme(Theme.default()) {
		Surface {
			PassphraseActions(true, {}, {}, {})
		}
	}
}
