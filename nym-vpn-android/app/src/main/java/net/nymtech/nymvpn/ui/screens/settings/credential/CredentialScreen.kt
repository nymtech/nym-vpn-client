package net.nymtech.nymvpn.ui.screens.settings.credential

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.Launch
import androidx.compose.material.icons.outlined.QrCodeScanner
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
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
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.functions.rememberImeState
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun CredentialScreen(appViewModel: AppViewModel, viewModel: CredentialViewModel = hiltViewModel()) {
	val snackbar = SnackbarController.current
	val imeState = rememberImeState()
	val scrollState = rememberScrollState()
	val padding = WindowInsets.systemBars.asPaddingValues()
	val context = LocalContext.current
	val navController = LocalNavController.current

	val error = viewModel.error.collectAsStateWithLifecycle()
	val success = viewModel.success.collectAsStateWithLifecycle(null)

	LaunchedEffect(success.value) {
		if (success.value == true) {
			navController.navigateAndForget(Route.Main())
		}
	}

	val requestPermissionLauncher = rememberLauncherForActivityResult(
		ActivityResultContracts.RequestPermission(),
	) { isGranted ->
		if (!isGranted) return@rememberLauncherForActivityResult snackbar.showMessage(context.getString(R.string.permission_required))
		navController.navigate(Route.CredentialScanner)
	}

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				show = false,
			),
		)
	}

	var credential by remember {
		mutableStateOf("")
	}

	LaunchedEffect(imeState.value) {
		if (imeState.value) {
			scrollState.animateScrollTo(scrollState.viewportSize)
		}
	}

	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(40.dp.scaledHeight(), Alignment.Bottom),
		modifier =
		Modifier
			.fillMaxSize()
			.imePadding()
			.verticalScroll(scrollState)
			.padding(horizontal = 24.dp.scaledWidth()).padding(padding),
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
			TextButton(onClick = { context.openWebUrl("https://nymvpn.com/") }) {
				Row(
					horizontalArrangement = Arrangement.spacedBy(5.dp.scaledWidth())
				){
					Text("Get credential", textAlign = TextAlign.Center)
					val icon = Icons.AutoMirrored.Outlined.Launch
					Icon(icon, icon.name, Modifier.size(20.dp.scaledHeight()))
				}

			}
		}
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(32.dp.scaledHeight(), Alignment.Top),
		) {
			CustomTextField(
				value = credential,
				onValueChange = {
					if (error.value != null) viewModel.resetError()
					credential = it
				},
				modifier = Modifier
					.width(358.dp.scaledWidth())
					.height(212.dp.scaledHeight()),
				supportingText = {
					if (error.value != null) {
						// TODO need a better way to determine this in the future
						Text(
							modifier = Modifier.fillMaxWidth(),
							text = if (error.value!!.contains("unique constraint violation")) {
								stringResource(R.string.credential_already_imported)
							} else {
								stringResource(R.string.credential_failed_message)
							},
							color = MaterialTheme.colorScheme.error,
						)
					}
				},
				isError = error.value != null,
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
							viewModel.onImportCredential(credential)
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
							requestPermissionLauncher.launch(android.Manifest.permission.CAMERA)
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
