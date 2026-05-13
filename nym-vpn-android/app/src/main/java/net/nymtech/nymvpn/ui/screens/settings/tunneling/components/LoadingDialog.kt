package net.nymtech.nymvpn.ui.screens.settings.tunneling.components

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun LoadingDialog() {
	Dialog(
		onDismissRequest = { },
		properties = DialogProperties(dismissOnBackPress = false, dismissOnClickOutside = false),
	) {
		Box(
			modifier = Modifier
				.sizeIn(minWidth = 120.dp.scaledHeight(), minHeight = 120.dp.scaledHeight())
				.background(
					color = MaterialTheme.colorScheme.primaryContainer,
					shape = MaterialTheme.shapes.medium,
				),
			contentAlignment = Alignment.Center,
		) {
			Column(
				modifier = Modifier.padding(16.dp.scaledHeight()),
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
					color = MaterialTheme.colorScheme.onBackground,
					textAlign = TextAlign.Center,
				)
			}
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, locale = "fr", widthDp = 120)
internal fun PreviewLoadingDialog() {
	NymVPNTheme(Theme.default()) {
		LoadingDialog()
	}
}
