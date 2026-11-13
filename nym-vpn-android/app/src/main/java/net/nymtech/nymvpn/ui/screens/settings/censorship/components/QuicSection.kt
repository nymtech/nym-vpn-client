package net.nymtech.nymvpn.ui.screens.settings.censorship.components

import android.content.res.Configuration
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
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
			BridgeModeSelector(bridgeMode, onBridgeModeSelected, Modifier.padding(top = 16.dp))
			Text(
				text = stringResource(bridgeMode.descriptionRes()),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.padding(top = 12.dp),
			)
			Text(
				text = stringResource(R.string.censorship_bridge_changes_apply),
				style = MaterialTheme.typography.bodyMedium,
				color = LocalNymColors.current.warning,
				modifier = Modifier.padding(top = 8.dp),
			)
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

private fun BridgeMode.titleRes() = when (this) {
	BridgeMode.OFF -> R.string.censorship_bridge_off_title
	BridgeMode.AUTO -> R.string.censorship_bridge_auto_title
	BridgeMode.ON -> R.string.censorship_bridge_on_title
}

private fun BridgeMode.descriptionRes() = when (this) {
	BridgeMode.OFF -> R.string.censorship_bridge_off_description
	BridgeMode.AUTO -> R.string.censorship_bridge_auto_description
	BridgeMode.ON -> R.string.censorship_bridge_on_description
}

private val bridgeModes = listOf(BridgeMode.OFF, BridgeMode.AUTO, BridgeMode.ON)

@Composable
private fun BridgeModeSelector(bridgeMode: BridgeMode, onBridgeModeSelected: (BridgeMode) -> Unit, modifier: Modifier = Modifier) {
	val index = bridgeModes.indexOf(bridgeMode)
	BoxWithConstraints(
		modifier = modifier
			.fillMaxWidth()
			.background(MaterialTheme.colorScheme.surfaceVariant, CircleShape)
			.padding(4.dp),
	) {
		val segmentWidth = maxWidth / bridgeModes.size
		val thumbOffset by animateDpAsState(segmentWidth * index, spring(dampingRatio = 0.78f, stiffness = Spring.StiffnessMediumLow), label = "bridgeModeThumb")
		Box(
			modifier = Modifier
				.offset(x = thumbOffset)
				.width(segmentWidth)
				.height(40.dp)
				.background(MaterialTheme.colorScheme.primary, CircleShape),
		)
		Row(modifier = Modifier.fillMaxWidth()) {
			bridgeModes.forEach { mode ->
				val selected = mode == bridgeMode
				val textColor by animateColorAsState(
					if (selected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onPrimaryContainer,
					label = "bridgeModeText",
				)
				Box(
					contentAlignment = Alignment.Center,
					modifier = Modifier
						.weight(1f)
						.height(40.dp)
						.selectable(
							selected = selected,
							interactionSource = remember { MutableInteractionSource() },
							indication = null,
							role = Role.RadioButton,
							onClick = { onBridgeModeSelected(mode) },
						),
				) {
					Text(stringResource(mode.titleRes()), style = MaterialTheme.typography.bodyMedium, color = textColor)
				}
			}
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
