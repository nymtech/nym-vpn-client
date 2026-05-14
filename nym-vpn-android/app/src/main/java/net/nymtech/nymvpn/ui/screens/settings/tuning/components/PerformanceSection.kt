package net.nymtech.nymvpn.ui.screens.settings.tuning.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun PerformanceSection(speed: String, latency: String, shape: Shape = RoundedCornerShape(8.dp)) {
	Card(
		shape = shape,
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
	) {
		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(16.dp),
			verticalArrangement = Arrangement.spacedBy(12.dp),
		) {
			Text(
				text = stringResource(R.string.mixnet_tuning_current_title),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)

			PerformanceRow(
				label = stringResource(R.string.mixnet_tuning_current_speed),
				value = speed,
			)

			HorizontalDivider(
				thickness = 0.5.dp,
				color = MaterialTheme.colorScheme.outline.copy(alpha = 0.3f),
			)

			PerformanceRow(
				label = stringResource(R.string.mixnet_tuning_current_latency),
				value = latency,
			)

			Text(
				text = stringResource(R.string.mixnet_tuning_current_description),
				style = MaterialTheme.typography.labelSmall,
				color = MaterialTheme.colorScheme.onBackground,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		}
	}
}

@Composable
private fun PerformanceRow(label: String, value: String) {
	Row(
		modifier = Modifier.fillMaxWidth(),
		horizontalArrangement = Arrangement.SpaceBetween,
		verticalAlignment = Alignment.CenterVertically,
	) {
		Text(
			text = label,
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
		)
		Text(
			text = value,
			style = MaterialTheme.typography.labelLarge,
			color = MaterialTheme.colorScheme.primary,
		)
	}
}

@Composable
@Preview
internal fun PreviewPerformanceSettingsSection() {
	NymVPNTheme(Theme.default()) {
		PerformanceSection(
			speed = "Up to 1 Mbps",
			latency = "At least 700 ms",
		)
	}
}
