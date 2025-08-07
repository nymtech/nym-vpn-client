package net.nymtech.nymvpn.ui.screens.main.modal

import android.content.Context
import android.content.Intent
import android.provider.Settings
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.OutlineStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun BatteryModal(showBatteryDialog: Boolean, onClickSettings: () -> Unit, onClickSkip: () -> Unit, onDismiss: () -> Unit) {

	Modal(
	show = showBatteryDialog,
	onDismiss = onDismiss,
	title = {
		Text(
			text = stringResource(R.string.battery_opt_title).uppercase(),
			color = MaterialTheme.colorScheme.onSurface,
			style = CustomTypography.labelHuge,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
		)
	},
	text = {
		Column(modifier = Modifier.fillMaxWidth()) {
			Text(
				text = stringResource(R.string.battery_opt_descr),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onSurface,
			)
			Row(
				horizontalArrangement = Arrangement.spacedBy(16.dp.scaledWidth(), Alignment.Start),
				verticalAlignment = Alignment.CenterVertically,
				modifier = Modifier
					.fillMaxWidth()
					.padding(top = 24.dp),
			) {
				MainStyledButton(
					onClick = onClickSettings,
					content = { Text(stringResource(R.string.settings).uppercase(), fontFamily = FontFamily(Font(R.font.lab_grotesque_mono))) },
					modifier = Modifier
						.weight(1f)
						.height(46.dp),
				)
				OutlineStyledButton(
					onClick = onClickSkip,
					content = {
						Text(
							stringResource(R.string.skip).uppercase(),
							style = MaterialTheme.typography.labelLarge,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
						)
					},
					backgroundColor = Color.Transparent,
					modifier = Modifier
						.weight(1f)
						.height(46.dp),
				)
			}
		}
	},
	confirmButton = {},
	)
}
