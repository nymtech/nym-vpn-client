package net.nymtech.nymvpn.ui.screens.main.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun ConnectionButton(
	connectionState: ConnectionState,
	isMnemonicStored: Boolean,
	onConnect: () -> Unit,
	onDisconnect: () -> Unit,
	modifier: Modifier = Modifier,
	snackbar: SnackbarController,
	navController: NavController,
) {
	val context = LocalContext.current
	val scope = rememberCoroutineScope()

	Box(modifier = modifier.padding(horizontal = 24.dp.scaledWidth())) {
		when (connectionState) {
			ConnectionState.Disconnected, ConnectionState.Offline -> MainStyledButton(
				testTag = Constants.CONNECT_TEST_TAG,
				onClick = {
					scope.launch {
						if (!isMnemonicStored) return@launch navController.goFromRoot(Route.Login)
						if (connectionState is ConnectionState.Offline) return@launch snackbar.showMessage(context.getString(R.string.no_internet))
						onConnect()
					}
				},
				content = {
					Text(stringResource(R.string.connect).uppercase(), style = CustomTypography.labelHuge, fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)))
				},
				modifier = Modifier.fillMaxWidth().height(56.dp.scaledHeight()),
			)
			ConnectionState.Disconnecting, is ConnectionState.Connecting, ConnectionState.WaitingForConnection -> MainStyledButton(
				onClick = onDisconnect,
				content = {
					Text(
						stringResource(R.string.stop).uppercase(),
						style = CustomTypography.labelHuge,
						color = MaterialTheme.colorScheme.background,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
					)
				},
				color = CustomColors.disconnect,
				modifier = Modifier.fillMaxWidth().height(56.dp.scaledHeight()),
			)
			ConnectionState.Connected -> MainStyledButton(
				testTag = Constants.DISCONNECT_TEST_TAG,
				onClick = onDisconnect,
				content = {
					Text(
						stringResource(R.string.disconnect).uppercase(),
						style = CustomTypography.labelHuge,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
					)
				},
				color = CustomColors.disconnect,
				modifier = Modifier.fillMaxWidth().height(56.dp.scaledHeight()),
			)
		}
	}
}
