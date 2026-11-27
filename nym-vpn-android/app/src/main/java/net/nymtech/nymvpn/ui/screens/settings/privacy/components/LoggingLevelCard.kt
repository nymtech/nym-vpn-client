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
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LoggingLevelCard(level: LoggingLevel, showRestartNote: Boolean, onLevelChange: (LoggingLevel) -> Unit) {
	Card(
		modifier = Modifier.fillMaxWidth(),
		shape = RoundedCornerShape(8.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
	) {
		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(16.dp),
			verticalArrangement = Arrangement.spacedBy(10.dp),
		) {
			Row(verticalAlignment = Alignment.CenterVertically) {
				// Decorative: the title beside it already names the card for TalkBack.
				Icon(
					Icons.Outlined.FindInPage,
					contentDescription = null,
					modifier = Modifier
						.padding(end = 16.dp.scaledWidth())
						.size(iconSize.scaledWidth()),
					tint = MaterialTheme.colorScheme.onSurfaceVariant,
				)
				Text(
					text = stringResource(R.string.logs_level_title),
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
			}
			// Full card width, so three labels fit on a narrow phone.
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
			// Changes with the selection; polite live regions let TalkBack read the new text.
			Text(
				text = stringResource(level.descriptionRes),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
			)
			// The core sets its log level once per process, so entering or leaving Debug needs a restart.
			if (showRestartNote) {
				Text(
					text = stringResource(R.string.privacy_error_reports_restart),
					style = MaterialTheme.typography.bodySmall,
					color = LocalNymColors.current.warning,
					modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
				)
			}
		}
	}
}
