package net.nymtech.nymvpn.ui.screens.details.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.colorLoad
import net.nymtech.nymvpn.util.extensions.colorPerformance
import net.nymtech.nymvpn.util.extensions.displayText
import net.nymtech.nymvpn.util.extensions.getScoreIcon
import net.nymtech.nymvpn.util.extensions.relativeTimeSpan
import nym_vpn_lib_types.Score

@Composable
fun DetailsSectionPerformance(score: Score?, load: Score?, uptime: Float?, lastUpdated: String?) {
	val items = buildList<Pair<String, @Composable () -> Unit>> {
		add(
			stringResource(R.string.details_overall_performance) to {
				Row(verticalAlignment = Alignment.CenterVertically) {
					val scoreIcon = getScoreIcon(score)
					if (score != null && scoreIcon != null) {
						Image(
							scoreIcon.first,
							contentDescription = scoreIcon.second,
							modifier = Modifier.size(16.dp),
						)
						Spacer(modifier = Modifier.width(6.dp))
						Text(
							text = score.displayText(),
							style = Typography.bodyMedium,
							color = score.colorPerformance(),
						)
					} else {
						Text(
							text = stringResource(R.string.not_applicable),
							style = Typography.bodyMedium,
							color = MaterialTheme.colorScheme.onBackground,
						)
					}
				}
			},
		)

		add(
			stringResource(R.string.details_server_load) to {
				Row(verticalAlignment = Alignment.CenterVertically) {
					Text(
						text = load?.displayText() ?: stringResource(R.string.not_applicable),
						style = Typography.bodyMedium,
						color = load?.colorLoad() ?: MaterialTheme.colorScheme.onBackground,
					)
				}
			},
		)

		add(
			stringResource(R.string.details_uptime) to {
				Text(
					text = uptime?.let { u -> "${u.toInt()}%" } ?: stringResource(R.string.not_applicable),
					style = Typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
			},
		)
	}

	InfoSection(
		titleResId = R.string.details_performance_title,
		items = items,
		bottomContent = {
			val relativeTimeSpan = lastUpdated?.let { relativeTimeSpan(it) } ?: stringResource(R.string.not_applicable)
			Text(
				text = stringResource(
					R.string.details_performance_calculated,
					relativeTimeSpan,
				),
				style = Typography.labelSmall,
				color = MaterialTheme.colorScheme.onBackground,
			)
		},
	)
}

@Composable
@PreviewLightDark
private fun PreviewDetailsSectionPerformance() {
	NymVPNTheme(Theme.default()) {
		Surface {
			DetailsSectionPerformance(
				score = Score.HIGH,
				load = Score.HIGH,
				uptime = 89f,
				lastUpdated = "September 11, 2025 at 13:31",
			)
		}
	}
}
