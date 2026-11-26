package net.nymtech.nymvpn.ui.screens.settings.privacy.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.FindInPage
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LoggingLevelCard(level: LoggingLevel, onLevelChange: (LoggingLevel) -> Unit) {
	Card(
		modifier = Modifier.fillMaxWidth(),
		shape = RoundedCornerShape(8.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
	) {
		Row(
			modifier = Modifier
				.fillMaxWidth()
				.padding(16.dp),
			verticalAlignment = Alignment.Top,
		) {
			// Decorative: the title beside it already names the card for TalkBack.
			Icon(
				Icons.Outlined.FindInPage,
				contentDescription = null,
				modifier = Modifier
					.padding(end = 16.dp.scaledWidth())
					.size(iconSize.scaledWidth()),
				tint = MaterialTheme.colorScheme.onSurfaceVariant,
			)
			Column(
				modifier = Modifier.weight(1f),
				verticalArrangement = Arrangement.spacedBy(10.dp),
			) {
				Text(
					text = stringResource(R.string.logs_level_title),
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
				SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
					LoggingLevel.entries.forEachIndexed { index, option ->
						SegmentedButton(
							selected = option == level,
							onClick = { onLevelChange(option) },
							shape = SegmentedButtonDefaults.itemShape(index = index, count = LoggingLevel.entries.size),
							colors = SegmentedButtonDefaults.colors(
								activeContainerColor = MaterialTheme.colorScheme.primary,
								activeContentColor = MaterialTheme.colorScheme.onPrimary,
								activeBorderColor = MaterialTheme.colorScheme.outline,
								inactiveContainerColor = Color.Transparent,
								inactiveContentColor = MaterialTheme.colorScheme.onBackground,
								inactiveBorderColor = MaterialTheme.colorScheme.outline,
							),
							icon = {},
							label = {
								Text(
									text = stringResource(option.labelRes),
									style = MaterialTheme.typography.bodyMedium,
								)
							},
						)
					}
				}
				Text(
					text = stringResource(level.descriptionRes),
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.onBackground,
				)
				// Only DEBUG needs a restart: the core reads its log level when it starts.
				if (level == LoggingLevel.DEBUG) {
					Text(
						text = stringResource(R.string.privacy_error_reports_restart),
						style = MaterialTheme.typography.bodySmall,
						color = LocalNymColors.current.warning,
					)
				}
			}
		}
	}
}
