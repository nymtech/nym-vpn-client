package net.nymtech.nymvpn.ui.screens.settings.credential

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.QrCodeScanner
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavController
import com.journeyapps.barcodescanner.ScanContract
import com.journeyapps.barcodescanner.ScanOptions
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.functions.rememberImeState
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.navigateNoBack
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun CredentialScreen(navController: NavController, appViewModel: AppViewModel, viewModel: CredentialViewModel = hiltViewModel()) {
	var credential by remember {
		mutableStateOf("")
	}

	var isCredentialError by remember {
		mutableStateOf(false)
	}

	val context = LocalContext.current
	val scope = rememberCoroutineScope()

	val imeState = rememberImeState()
	val scrollState = rememberScrollState()

	fun onAddCredential() {
		scope.launch {
			viewModel.onImportCredential(credential).onSuccess { _ ->
				appViewModel.showSnackbarMessage(context.getString(R.string.credential_successful))
				navController.navigateNoBack(NavItem.Main.route)
			}.onFailure {
				isCredentialError = true
			}
		}
	}

	val scanLauncher =
		rememberLauncherForActivityResult(
			contract = ScanContract(),
			onResult = {
				if (it.contents != null) {
					credential = ""
					credential = it.contents
					onAddCredential()
				}
			},
		)

	LaunchedEffect(imeState.value) {
		if (imeState.value) {
			scrollState.animateScrollTo(scrollState.viewportSize)
		}
	}

	fun launchQrScanner() {
		val scanOptions = ScanOptions()
		scanOptions.setDesiredBarcodeFormats(ScanOptions.QR_CODE)
		scanOptions.setOrientationLocked(true)
		scanOptions.setPrompt(
			context.getString(R.string.scan_nym_vpn_credential),
		)
		scanOptions.setBeepEnabled(false)
		scanLauncher.launch(scanOptions)
	}

	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(40.dp.scaledHeight(), Alignment.Bottom),
		modifier =
		Modifier
			.fillMaxSize()
			.imePadding()
			.verticalScroll(scrollState)
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		Image(
			painter = painterResource(id = R.drawable.login),
			contentDescription = stringResource(id = R.string.login),
			contentScale = ContentScale.None,
			modifier =
			Modifier
				.width(80.dp)
				.height(80.dp),
		)
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Top),
			modifier =
			Modifier
				.padding(
					horizontal = 24.dp.scaledWidth(),
					vertical = 24.dp.scaledHeight(),
				),
		) {
			Text(
				text = stringResource(id = R.string.welcome_exclaim),
				style = MaterialTheme.typography.headlineSmall,
				color = MaterialTheme.colorScheme.onBackground,
			)
			Text(
				text = stringResource(id = R.string.credential_message),
				style = MaterialTheme.typography.bodyLarge,
				color = MaterialTheme.colorScheme.onSurfaceVariant,
				textAlign = TextAlign.Center,
			)
			Text(
				text = stringResource(id = R.string.credential_disclaimer),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onSurfaceVariant,
				textAlign = TextAlign.Center,
			)
		}
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(32.dp.scaledHeight(), Alignment.Top),
		) {
			CustomTextField(
				value = credential,
				onValueChange = {
					if (isCredentialError) isCredentialError = false
					credential = it
				},
				modifier = Modifier
					.width(358.dp.scaledWidth())
					.height(212.dp.scaledHeight()),
				supportingText = {
					if (isCredentialError) {
						Text(
							modifier = Modifier.fillMaxWidth(),
							text = stringResource(id = R.string.credential_failed_message),
							color = MaterialTheme.colorScheme.error,
						)
					}
				},
				isError = isCredentialError,
				label = { Text(text = stringResource(id = R.string.credential_label)) },
				textStyle = MaterialTheme.typography.bodyMedium.copy(
					color = MaterialTheme.colorScheme.onSurface,
				),
			)
			Row(
				horizontalArrangement = Arrangement.spacedBy(16.dp.scaledWidth(), Alignment.CenterHorizontally),
				modifier = Modifier
					.fillMaxWidth()
					.padding(bottom = 24.dp.scaledHeight()),
			) {
				Box(modifier = Modifier.width(286.dp.scaledWidth())) {
					MainStyledButton(
						Constants.LOGIN_TEST_TAG,
						onClick = {
							onAddCredential()
						},
						content = {
							Text(
								stringResource(id = R.string.add_credential),
								style = CustomTypography.labelHuge,
							)
						},
						color = MaterialTheme.colorScheme.primary,
					)
				}
				Box(modifier = Modifier.width(56.dp.scaledWidth())) {
					MainStyledButton(
						onClick = {
							launchQrScanner()
						},
						content = {
							val icon = Icons.Outlined.QrCodeScanner
							Icon(icon, icon.name, modifier = Modifier.size(iconSize.scaledWidth()))
						},
						color = MaterialTheme.colorScheme.primary,
					)
				}
			}
		}
	}
}
