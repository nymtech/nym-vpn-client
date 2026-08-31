package net.nymtech.nymvpn.ui.screens.settings.tuning.components

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
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
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import nym_vpn_lib_types.ContinuousTrafficSendingRate

private val RATES = ContinuousTrafficSendingRate.entries

@Composable
fun SendTrafficSection(trafficEnabled: Boolean, onTrafficEnable: (enabled: Boolean) -> Unit, trafficRate: ContinuousTrafficSendingRate, onTrafficRateChange: (ContinuousTrafficSendingRate) -> Unit) {
	val interactionSource = remember { MutableInteractionSource() }

	Card(
		shape = RoundedCornerShape(8.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
	) {
		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(16.dp),
			verticalArrangement = Arrangement.spacedBy(16.dp),
		) {
			Row(
				modifier = Modifier.fillMaxWidth(),
				horizontalArrangement = Arrangement.SpaceBetween,
				verticalAlignment = Alignment.CenterVertically,
			) {
				Text(
					text = stringResource(R.string.mixnet_tuning_traffic_title),
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					maxLines = 2,
					overflow = TextOverflow.Ellipsis,
					modifier = Modifier.weight(1f),
				)
				Spacer(modifier = Modifier.width(8.dp))
				ScaledSwitch(
					checked = trafficEnabled,
					onClick = { onTrafficEnable(it) },
				)
			}

			Text(
				text = stringResource(if (trafficEnabled) R.string.mixnet_tuning_traffic_on_description else R.string.mixnet_tuning_traffic_warning),
				style = MaterialTheme.typography.bodySmall,
				color = if (trafficEnabled) MaterialTheme.colorScheme.onBackground else LocalNymColors.current.warning,
			)

			Column(modifier = Modifier.fillMaxWidth()) {
				SliderRangeLabels()
				DiscreteTuningSlider(
					items = RATES,
					selected = trafficRate,
					onSelectedChange = onTrafficRateChange,
					interactionSource = interactionSource,
				)
			}

			DiscreteRateLabels(
				labels = listOf(
					stringResource(R.string.mixnet_tuning_traffic_on_low),
					stringResource(R.string.mixnet_tuning_traffic_on_balanced),
					stringResource(R.string.mixnet_tuning_traffic_on_high),
				),
				selectedIndex = RATES.indexOf(trafficRate),
			)
		}
	}
}

@Composable
@Preview
internal fun PreviewMSendTrafficSection() {
	NymVPNTheme(Theme.default()) {
		SendTrafficSection(
			trafficEnabled = true,
			onTrafficEnable = {},
			trafficRate = ContinuousTrafficSendingRate.MS20,
			onTrafficRateChange = {},
		)
	}
}
