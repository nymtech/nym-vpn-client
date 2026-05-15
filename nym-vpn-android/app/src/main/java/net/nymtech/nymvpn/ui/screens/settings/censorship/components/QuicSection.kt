package net.nymtech.nymvpn.ui.screens.settings.censorship.components

import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.openWebUrl

@Composable
fun QuicSection(quicEnabled: Boolean, onQuicEnabledEnable: (enabled: Boolean) -> Unit, shape: Shape = RoundedCornerShape(8.dp), modifier: Modifier = Modifier) {
	val context = LocalContext.current
	val interactionSource = remember { MutableInteractionSource() }

	Card(
		modifier = modifier,
		shape = shape,
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
	) {
		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(horizontal = 16.dp, vertical = 16.dp),
		) {
			Row(
				modifier = Modifier.fillMaxWidth(),
				horizontalArrangement = Arrangement.SpaceBetween,
				verticalAlignment = Alignment.CenterVertically,
			) {
				Text(
					text = stringResource(R.string.censorship_quic_title),
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
				ScaledSwitch(
					checked = quicEnabled,
					onClick = { onQuicEnabledEnable(it) },
				)
			}
			Text(
				text = stringResource(R.string.censorship_quic_description),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier
					.fillMaxWidth()
					.padding(top = 16.dp),
				textAlign = TextAlign.Justify,
			)
			Row(
				horizontalArrangement = Arrangement.spacedBy(2.dp, Alignment.Start),
				modifier = Modifier
					.fillMaxWidth()
					.padding(top = 16.dp)
					.clickable(
						interactionSource = interactionSource,
						indication = null,
					) {
						context.openWebUrl(context.getString(R.string.censorship_quic_link))
					},
			) {
				Text(
					text = stringResource(R.string.censorship_learn_more_link_text),
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					style = MaterialTheme.typography.bodyMedium.copy(
						textDecoration = TextDecoration.Underline,
					),
				)
				Icon(
					Icons.AutoMirrored.Outlined.OpenInNew,
					stringResource(R.string.go),
					Modifier
						.size(16.dp)
						.align(Alignment.CenterVertically),
				)
			}
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewQuicSection() {
	NymVPNTheme(Theme.default()) {
		QuicSection(true, {}, modifier = Modifier.fillMaxWidth())
	}
}
