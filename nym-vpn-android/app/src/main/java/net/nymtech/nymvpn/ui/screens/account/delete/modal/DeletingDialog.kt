package net.nymtech.nymvpn.ui.screens.account.delete.modal

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun DeletingDialog(show: Boolean) {
	Modal(
		show = show,
		onDismiss = {},
		title = {
			Text(
				text = stringResource(R.string.delete_account_dialog_progress_title),
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
					text = stringResource(R.string.delete_account_dialog_progress_descr),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground,
				)
			}
		},
		confirmButton = {},
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewDeletingDialog() {
	NymVPNTheme(Theme.default()) {
		DeletingDialog(show = true)
	}
}
