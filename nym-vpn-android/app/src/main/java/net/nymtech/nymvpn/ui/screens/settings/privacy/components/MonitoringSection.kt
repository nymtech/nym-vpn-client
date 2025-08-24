package net.nymtech.nymvpn.ui.screens.settings.privacy.components

import android.content.Context
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.util.extensions.openWebUrl

@Composable
fun MonitoringSection(
	sentryEnabled: Boolean,
	onMonitoringEnable: (enabled: Boolean) -> Unit,
	context: Context,
	shape: Shape = RoundedCornerShape(8.dp),
) {
	val interactionSource = remember { MutableInteractionSource() }

	Card(
		modifier = Modifier.fillMaxWidth(),
		shape = shape,
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
	) {
		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(horizontal = 16.dp, vertical = 16.dp),
		) {
			Row(
				modifier = Modifier.fillMaxWidth(),
				horizontalArrangement = Arrangement.SpaceBetween,
				verticalAlignment = Alignment.CenterVertically,
			) {
				Column {
					Text(
						text = stringResource(R.string.title_error_reports),
						style = MaterialTheme.typography.titleMedium,
					)
					Text(
						text = stringResource(R.string.restart_error_reports),
						style = MaterialTheme.typography.bodySmall,
						color = CustomColors.warning,
					)
				}
				ScaledSwitch(
					checked = sentryEnabled,
					onClick = { onMonitoringEnable(it) },
				)
			}

			Text(
				text = stringResource(R.string.descr_error_reports),
				style = MaterialTheme.typography.bodyMedium,
				modifier = Modifier
					.fillMaxWidth()
					.padding(top = 16.dp),
				textAlign = TextAlign.Justify,
			)

			Box(
				contentAlignment = Alignment.Center,
				modifier = Modifier
					.fillMaxWidth()
					.padding(top = 16.dp)
					.clickable(
						interactionSource = interactionSource,
						indication = null,
					) {
						context.openWebUrl(context.getString(R.string.link_error_reports))
					},
			) {
				Text(
					text = stringResource(R.string.link_error_reports_text),
					style = MaterialTheme.typography.bodyMedium.copy(
						textDecoration = TextDecoration.Underline,
					),
					modifier = Modifier.fillMaxWidth(),
				)
			}
		}
	}
}
