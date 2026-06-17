package net.nymtech.nymvpn.ui.screens.main.modal

import android.content.Context
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.screens.main.components.ModeModalBody
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.openWebUrl

@Composable
fun ShowInfoModal(context: Context, showInfoDialog: Boolean, onDismiss: () -> Unit) {
	Modal(
		show = showInfoDialog,
		onDismiss = { onDismiss() },
		title = {
			Text(
				text = stringResource(R.string.mode_selection),
				style = CustomTypography.labelHuge,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		text = {
			ModeModalBody(
				onClick = { context.openWebUrl(context.getString(R.string.mode_support_link)) },
			)
		},
	)
}
