package net.nymtech.nymvpn.ui.screens.settings.censorship.components

import android.content.res.Configuration
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import nym_vpn_lib_types.BridgeMode
import nym_vpn_lib_types.BridgeTransport

@Composable
fun TransportPreferenceSection(
	bridgeMode: BridgeMode,
	transportPreference: BridgeTransport?,
	onTransportPreferenceSelected: (BridgeTransport?) -> Unit,
	shape: Shape = RoundedCornerShape(8.dp),
	modifier: Modifier = Modifier,
) {
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
				text = stringResource(R.string.censorship_transport_title),
				style = MaterialTheme.typography.titleMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
			Text(
				text = stringResource(R.string.censorship_transport_description),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.padding(top = 12.dp),
			)
			TransportPreferenceSelector(transportPreference, onTransportPreferenceSelected, Modifier.padding(top = 16.dp))
			Text(
				text = stringResource(transportPreference.descriptionRes()),
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
			if (bridgeMode == BridgeMode.AUTO && transportPreference != null) {
				Text(
					text = stringResource(R.string.censorship_transport_auto_mode_disclaimer),
					style = MaterialTheme.typography.bodyMedium,
					color = LocalNymColors.current.warning,
					modifier = Modifier.padding(top = 8.dp),
				)
			}
		}
	}
}

private fun BridgeTransport?.titleRes() = when (this) {
	null -> R.string.censorship_transport_auto_title
	BridgeTransport.QUIC -> R.string.censorship_transport_quic_title
	BridgeTransport.TLS -> R.string.censorship_transport_tls_title
	BridgeTransport.SSH -> R.string.censorship_transport_ssh_title
}

private fun BridgeTransport?.descriptionRes() = when (this) {
	null -> R.string.censorship_transport_auto_description
	BridgeTransport.QUIC -> R.string.censorship_transport_quic_description
	BridgeTransport.TLS -> R.string.censorship_transport_tls_description
	BridgeTransport.SSH -> R.string.censorship_transport_ssh_description
}

private val transportPreferences: List<BridgeTransport?> = listOf(null, BridgeTransport.QUIC, BridgeTransport.TLS, BridgeTransport.SSH)

@Composable
private fun TransportPreferenceSelector(transportPreference: BridgeTransport?, onTransportPreferenceSelected: (BridgeTransport?) -> Unit, modifier: Modifier = Modifier) {
	val index = transportPreferences.indexOf(transportPreference)
	BoxWithConstraints(
		modifier = modifier
			.fillMaxWidth()
			.background(MaterialTheme.colorScheme.surfaceVariant, CircleShape)
			.padding(4.dp),
	) {
		val segmentWidth = maxWidth / transportPreferences.size
		val thumbOffset by animateDpAsState(segmentWidth * index, spring(dampingRatio = 0.78f, stiffness = Spring.StiffnessMediumLow), label = "transportPreferenceThumb")
		Box(
			modifier = Modifier
				.offset(x = thumbOffset)
				.width(segmentWidth)
				.height(40.dp)
				.background(MaterialTheme.colorScheme.primary, CircleShape),
		)
		Row(modifier = Modifier.fillMaxWidth()) {
			transportPreferences.forEach { preference ->
				val selected = preference == transportPreference
				val textColor by animateColorAsState(
					if (selected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onPrimaryContainer,
					label = "transportPreferenceText",
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
							onClick = { onTransportPreferenceSelected(preference) },
						),
				) {
					Text(stringResource(preference.titleRes()), style = MaterialTheme.typography.bodyMedium, color = textColor)
				}
			}
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewTransportPreferenceSection() {
	NymVPNTheme(Theme.default()) {
		TransportPreferenceSection(BridgeMode.AUTO, null, {}, modifier = Modifier.fillMaxWidth())
	}
}
