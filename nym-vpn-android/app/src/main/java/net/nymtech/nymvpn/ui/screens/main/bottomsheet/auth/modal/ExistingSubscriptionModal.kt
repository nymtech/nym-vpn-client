package net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.modal

import android.content.res.Configuration
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.TransparentButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun ExistingSubscriptionModal(showSubscriptionDialog: Boolean, onClickLogin: () -> Unit, onClickCancel: () -> Unit, onDismiss: () -> Unit) {
	Modal(
		show = showSubscriptionDialog,
		onDismiss = onDismiss,
		title = {
			Text(
				text = stringResource(R.string.account_subscription_title),
				style = CustomTypography.labelHuge,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
		},
		text = {
			Text(
				stringResource(R.string.account_subscription_description),
				textAlign = TextAlign.Center,
				style = MaterialTheme.typography.bodyMedium,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
		},
		confirmButton = {
			MainStyledButton(
				onClick = onClickLogin,
				content = {
					Text(
						stringResource(R.string.account_subscription_button),
						style = MaterialTheme.typography.titleMedium,
					)
				},
				modifier = Modifier.fillMaxWidth()
					.height(48.dp.scaledHeight()),
				shape = RoundedCornerShape(12.dp),
			)
		},
		dismissButton = {
			TransparentButton(
				onClick = onClickCancel,
				content = {
					Text(
						stringResource(R.string.close),
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
	)
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun BatteryModalPreview() {
	NymVPNTheme(Theme.default()) {
		ExistingSubscriptionModal(true, {}, {}, {})
	}
}
