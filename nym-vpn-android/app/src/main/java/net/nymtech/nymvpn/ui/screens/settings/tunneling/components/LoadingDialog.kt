package net.nymtech.nymvpn.ui.screens.settings.tunneling.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun LoadingDialog() {
	Dialog(
		onDismissRequest = { },
		properties = DialogProperties(dismissOnBackPress = false, dismissOnClickOutside = false),
	) {
		Box(
			modifier = Modifier
				.size(120.dp.scaledHeight())
				.background(
					color = MaterialTheme.colorScheme.surface,
					shape = MaterialTheme.shapes.medium,
				),
			contentAlignment = Alignment.Center,
		) {
			Column(
				horizontalAlignment = Alignment.CenterHorizontally,
				verticalArrangement = Arrangement.Center,
			) {
				CircularProgressIndicator(
					modifier = Modifier.size(48.dp.scaledHeight()),
					color = MaterialTheme.colorScheme.primary,
				)
				Spacer(modifier = Modifier.height(16.dp.scaledHeight()))
				Text(
					text = stringResource(R.string.loading),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onSurface,
				)
			}
		}
	}
}
