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
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import nym_vpn_lib_types.BackgroundCoverTrafficRate

private val RATES = BackgroundCoverTrafficRate.entries

@Composable
fun BackgroundCoverTrafficSection(enabled: Boolean, onEnable: (enabled: Boolean) -> Unit, rate: BackgroundCoverTrafficRate, onRateChange: (BackgroundCoverTrafficRate) -> Unit) {
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
					text = stringResource(R.string.mixnet_tuning_traffic_off_title),
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					maxLines = 2,
					overflow = TextOverflow.Ellipsis,
					modifier = Modifier.weight(1f),
				)
				Spacer(modifier = Modifier.width(8.dp))
				ScaledSwitch(
					checked = enabled,
					onClick = { onEnable(it) },
				)
			}

			Text(
				text = stringResource(R.string.mixnet_tuning_traffic_off_description),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onBackground,
			)

			Column(modifier = Modifier.fillMaxWidth()) {
				SliderRangeLabels()
				DiscreteTuningSlider(
					items = RATES,
					selected = rate,
					onSelectedChange = onRateChange,
					interactionSource = interactionSource,
				)
			}

			DiscreteRateLabels(
				labels = listOf(
					stringResource(R.string.mixnet_tuning_traffic_off_low),
					stringResource(R.string.mixnet_tuning_traffic_off_balanced),
					stringResource(R.string.mixnet_tuning_traffic_off_medium),
					stringResource(R.string.mixnet_tuning_traffic_off_high),
				),
				selectedIndex = RATES.indexOf(rate),
			)
		}
	}
}

@Composable
@Preview
internal fun PreviewBackgroundCoverTrafficSection() {
	NymVPNTheme(Theme.default()) {
		BackgroundCoverTrafficSection(
			enabled = false,
			onEnable = {},
			rate = BackgroundCoverTrafficRate.MS40,
			onRateChange = {},
		)
	}
}
