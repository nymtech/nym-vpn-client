package net.nymtech.nymvpn.ui.screens.account.info.modal

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.TransparentButton

@Composable
fun AutologinLoadingDialog(onCancel: () -> Unit) {
	Modal(
		show = true,
		onDismiss = onCancel,
		title = {
			Text(
				text = stringResource(R.string.account_info_autologin_fetching),
				style = MaterialTheme.typography.titleMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
		},
		text = {
			Row(
				verticalAlignment = Alignment.CenterVertically,
				horizontalArrangement = Arrangement.Center,
				modifier = Modifier.fillMaxWidth(),
			) {
				CircularProgressIndicator(
					modifier = Modifier.padding(end = 12.dp).size(24.dp),
					strokeWidth = 2.dp,
				)
				Text(
					text = stringResource(R.string.loading),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground,
				)
			}
		},
		confirmButton = {},
		dismissButton = {
			TransparentButton(
				onClick = onCancel,
				content = {
					Text(
						text = stringResource(R.string.cancel),
						style = MaterialTheme.typography.labelLarge,
					)
				},
				modifier = Modifier.fillMaxWidth().height(40.dp),
			)
		},
	)
}
