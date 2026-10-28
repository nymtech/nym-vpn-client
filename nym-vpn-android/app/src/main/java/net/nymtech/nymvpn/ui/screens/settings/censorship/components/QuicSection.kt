package net.nymtech.nymvpn.ui.screens.settings.censorship.components

import android.content.res.Configuration
import androidx.annotation.StringRes
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.openWebUrl
import nym_vpn_lib_types.BridgeMode

@Composable
fun QuicSection(bridgeMode: BridgeMode, onBridgeModeSelected: (BridgeMode) -> Unit, showBridgeFallback: Boolean = false, shape: Shape = RoundedCornerShape(8.dp), modifier: Modifier = Modifier) {
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
			Text(
				text = stringResource(R.string.censorship_quic_title),
				style = MaterialTheme.typography.titleMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
			Text(
				text = stringResource(R.string.censorship_bridge_changes_apply),
				style = MaterialTheme.typography.bodyMedium,
				color = LocalNymColors.current.warning,
				modifier = Modifier.padding(top = 8.dp),
			)
			BridgeModeOption(R.string.censorship_bridge_off_title, R.string.censorship_bridge_off_description, bridgeMode == BridgeMode.OFF) { onBridgeModeSelected(BridgeMode.OFF) }
			BridgeModeOption(R.string.censorship_bridge_auto_title, R.string.censorship_bridge_auto_description, bridgeMode == BridgeMode.AUTO) { onBridgeModeSelected(BridgeMode.AUTO) }
			BridgeModeOption(R.string.censorship_bridge_on_title, R.string.censorship_bridge_on_description, bridgeMode == BridgeMode.ON) { onBridgeModeSelected(BridgeMode.ON) }
			if (showBridgeFallback && bridgeMode != BridgeMode.OFF) {
				Text(
					text = stringResource(R.string.censorship_quic_bridge_fallback),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground,
					modifier = Modifier
						.fillMaxWidth()
						.padding(top = 16.dp),
				)
			}
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
private fun BridgeModeOption(@StringRes title: Int, @StringRes description: Int, selected: Boolean, onClick: () -> Unit) {
	Row(
		modifier = Modifier
			.fillMaxWidth()
			.padding(top = 8.dp)
			.selectable(selected = selected, role = Role.RadioButton, onClick = onClick),
		verticalAlignment = Alignment.CenterVertically,
	) {
		RadioButton(selected = selected, onClick = null)
		Column(modifier = Modifier.padding(start = 12.dp)) {
			Text(
				text = stringResource(title),
				style = MaterialTheme.typography.titleSmall,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
			Text(
				text = stringResource(description),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onBackground,
			)
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewQuicSection() {
	NymVPNTheme(Theme.default()) {
		QuicSection(BridgeMode.AUTO, {}, modifier = Modifier.fillMaxWidth())
	}
}
