package net.nymtech.nymvpn.ui.screens.account.passphrase.components

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun PassphraseActions(show: Boolean, onCopyClick: () -> Unit, onSaveClick: () -> Unit) {
	if (show) {
		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 8.dp),
		) {
			Row(
				modifier = Modifier.fillMaxWidth(),
				verticalAlignment = Alignment.CenterVertically,
				horizontalArrangement = Arrangement.Center,
			) {
				TextButton(onClick = onCopyClick, contentPadding = PaddingValues(0.dp)) {
					Text(
						stringResource(R.string.passphrase_copy),
						style = MaterialTheme.typography.labelLarge,
						color = MaterialTheme.colorScheme.primary,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				}

				VerticalDivider(
					modifier = Modifier
						.height(18.dp)
						.padding(horizontal = 12.dp)
						.width(1.dp),
					color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.25f),
				)

				TextButton(onClick = onSaveClick, contentPadding = PaddingValues(0.dp)) {
					Text(
						stringResource(R.string.passphrase_save),
						style = MaterialTheme.typography.labelLarge,
						color = MaterialTheme.colorScheme.primary,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				}
			}

			Column(
				verticalArrangement = Arrangement.spacedBy(6.dp),
				horizontalAlignment = Alignment.CenterHorizontally,
				modifier = Modifier
					.fillMaxWidth()
					.padding(8.dp),
			) {
				Text(
					text = stringResource(R.string.passphrase_lose),
					style = MaterialTheme.typography.labelLarge,
					color = MaterialTheme.colorScheme.onSurface,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					modifier = Modifier.fillMaxWidth(),
					textAlign = TextAlign.Center,
				)
				Text(
					text = stringResource(R.string.passphrase_never),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.9f),
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					modifier = Modifier.fillMaxWidth(),
					textAlign = TextAlign.Center,
				)
			}
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewPassphraseActions() {
	NymVPNTheme(Theme.default()) {
		PassphraseActions(true, {}, {})
	}
}
