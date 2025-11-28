package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun LogoutSection(isMnemonicStored: Boolean, onLogoutClick: () -> Unit) {
	if (isMnemonicStored) {
		val interactionSource = remember { MutableInteractionSource() }
		Card(
			modifier = Modifier.fillMaxWidth()
				.border(
					width = 1.dp,
					color = CustomColors.buttonRedTransparentBorder,
					shape = RoundedCornerShape(8.dp),
				),
			shape = RoundedCornerShape(8.dp),
			colors = CardDefaults.cardColors(containerColor = CustomColors.buttonRedTransparent),
		) {
			Box(
				contentAlignment = Alignment.Center,
				modifier =
				Modifier
					.clickable(
						interactionSource = interactionSource,
						indication = null,
					) {
						onLogoutClick.invoke()
					}
					.fillMaxWidth().height(IntrinsicSize.Min),
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier.fillMaxSize(),
				) {
					Row(
						verticalAlignment = Alignment.CenterVertically,
						modifier = Modifier
							.weight(1f, false)
							.padding(vertical = 8.dp.scaledHeight())
							.fillMaxSize()
							.padding(end = 4.dp.scaledWidth()),
					) {
						Column(
							horizontalAlignment = Alignment.Start,
							verticalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterVertically),
							modifier = Modifier
								.fillMaxWidth()
								.padding(vertical = 16.dp.scaledHeight(), horizontal = 24.dp),
						) {
							Text(
								stringResource(R.string.log_out),
								style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
							)
						}
					}
				}
			}
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewLogoutSection() {
	NymVPNTheme(Theme.default()) {
		LogoutSection(true) {}
	}
}
