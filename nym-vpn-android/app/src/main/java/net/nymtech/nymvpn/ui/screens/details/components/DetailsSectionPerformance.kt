package net.nymtech.nymvpn.ui.screens.details.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.capitalizeFirstLowerRest
import net.nymtech.nymvpn.util.extensions.colorLoad
import net.nymtech.nymvpn.util.extensions.colorPerformance
import net.nymtech.nymvpn.util.extensions.formatUtcString
import net.nymtech.nymvpn.util.extensions.getScoreIcon
import nym_vpn_lib_types.Score

@Composable
fun DetailsSectionPerformance(score: Score?, load: Score?, uptime: Float?, lastUpdated: String?) {
	val items = buildList<Pair<String, @Composable () -> Unit>> {
		score?.let { s ->
			add(
				stringResource(R.string.details_overall_performance) to {
					Row(verticalAlignment = Alignment.CenterVertically) {
						val scoreIcon = getScoreIcon(s)
						Image(
							scoreIcon.first,
							contentDescription = scoreIcon.second,
							modifier = Modifier.size(16.dp),
						)
						Spacer(modifier = Modifier.width(6.dp))
						Text(
							text = s.name.capitalizeFirstLowerRest(),
							style = Typography.bodyMedium,
							color = s.colorPerformance(),
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						)
					}
				},
			)
		}

		load?.let { l ->
			add(
				stringResource(R.string.details_server_load) to {
					Row(verticalAlignment = Alignment.CenterVertically) {
						Text(
							text = l.name.capitalizeFirstLowerRest(),
							style = Typography.bodyMedium,
							color = l.colorLoad(),
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						)
					}
				},
			)
		}

		add(
			stringResource(R.string.details_uptime) to {
				Text(
					text = uptime?.let { u -> "${u.toInt()}%" } ?: stringResource(R.string.not_applicable),
					style = Typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)
			},
		)
	}

	InfoSection(
		items = items,
		bottomContent = lastUpdated?.let {
			{
				val formattedDate = formatUtcString(it)
				Text(
					text = stringResource(
						R.string.details_performance_calculated,
						formattedDate,
					),
					style = Typography.labelSmall,
					color = MaterialTheme.colorScheme.outline,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)
			}
		},
	)
}
