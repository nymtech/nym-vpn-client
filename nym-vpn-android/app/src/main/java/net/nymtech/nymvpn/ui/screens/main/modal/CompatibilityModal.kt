package net.nymtech.nymvpn.ui.screens.main.modal

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun CompatibilityModal(showCompatibilityDialog: Boolean, onDismiss: () -> Unit, onConfirmClick: () -> Unit) {
	Modal(
		show = showCompatibilityDialog,
		onDismiss = onDismiss,
		title = {
			Text(
				text = stringResource(R.string.update_required),
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				style = CustomTypography.labelHuge,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		text = {
			Text(
				stringResource(R.string.app_update_required),
				textAlign = TextAlign.Center,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		confirmButton = {
			MainStyledButton(
				onClick = onConfirmClick,
				textColor = Color.Black,
				content = { Text(stringResource(R.string.update), fontFamily = FontFamily(Font(R.font.lab_grotesque_regular))) },
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
	)
}
