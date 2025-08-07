package net.nymtech.nymvpn.ui.screens.main.modal

import android.content.Context
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
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
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun CompatibilityModal(showCompatibilityDialog: Boolean, onDismiss: () -> Unit, confirmClick: () -> Unit) {

	Modal(
		show = showCompatibilityDialog,
		onDismiss = onDismiss,
		title = {
			Text(
				text = stringResource(R.string.update_required).uppercase(),
				color = MaterialTheme.colorScheme.onSurface,
				style = CustomTypography.labelHuge,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
			)
		},
		text = {
			Column(verticalArrangement = Arrangement.spacedBy(16.dp.scaledHeight())) {
				Row(
					horizontalArrangement = Arrangement.spacedBy(10.dp.scaledWidth(), Alignment.CenterHorizontally),
					verticalAlignment = Alignment.CenterVertically,
				) {
					Text(
						text = stringResource(R.string.app_update_required),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onSurface,
					)
				}
			}
		},
		confirmButton = {
			MainStyledButton(
				onClick = confirmClick,
				content = { Text(stringResource(R.string.update).uppercase(), fontFamily = FontFamily(Font(R.font.lab_grotesque_mono))) },
				modifier = Modifier
					.fillMaxWidth()
					.height(56.dp.scaledHeight()),
			)
		},
	)
}
