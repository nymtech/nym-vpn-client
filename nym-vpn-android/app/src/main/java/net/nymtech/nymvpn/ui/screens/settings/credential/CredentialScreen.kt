package net.nymtech.nymvpn.ui.screens.settings.credential

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.ClickableText
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Refresh
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.animations.SpinningIcon
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.functions.rememberImeState
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.screens.settings.credential.components.CredentialModalBody
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun CredentialScreen(appUiState: AppUiState, appViewModel: AppViewModel, viewModel: CredentialViewModel = hiltViewModel()) {
	val snackbar = SnackbarController.current
	val imeState = rememberImeState()
	val scrollState = rememberScrollState()
	val padding = WindowInsets.systemBars.asPaddingValues()
	val context = LocalContext.current
	val navController = LocalNavController.current
	val keyboardController = LocalSoftwareKeyboardController.current

	val success = viewModel.success.collectAsStateWithLifecycle(null)
	val showMaxDevicesModal = viewModel.showMaxDevicesModal.collectAsStateWithLifecycle(null)
	var showModal by remember { mutableStateOf(false) }
	var loading by remember { mutableStateOf(false) }

	LaunchedEffect(success.value) {
		loading = false
		if (success.value == true) navController.navigateAndForget(Route.Main())
		if (success.value == false && showMaxDevicesModal.value == true) showModal = true
	}

	val onDismiss = {
		showModal = false
	}

	Modal(show = showModal, onDismiss = {
		onDismiss()
	}, title = {
		Text(
			text = stringResource(R.string.max_devices_reached_title),
			color = MaterialTheme.colorScheme.onSurface,
			style = CustomTypography.labelHuge,
			textAlign = TextAlign.Center,
		)
	}, text = {
		CredentialModalBody {
			appUiState.managerState.accountLinks?.signIn?.let {
				context.openWebUrl(it)
				onDismiss()
			}
		}
	}, confirmButton = {
		Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center) {
			TextButton(
				onClick = {
					onDismiss()
				},
				content = {
					Text(stringResource(id = R.string.close), style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.primary)
				},
			)
		}
	})

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

	var mnemonic by remember {
		mutableStateOf("")
	}

	val onSubmit = {
		loading = true
		viewModel.onMnemonicImport(mnemonic)
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
		Icon(imageVector = ImageVector.vectorResource(R.drawable.app_label), contentDescription = stringResource(id = R.string.login))
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
				text = stringResource(id = R.string.recovery_phrase_message),
				style = MaterialTheme.typography.bodyLarge,
				color = MaterialTheme.colorScheme.onSurface,
				textAlign = TextAlign.Center,
			)
			Text(
				text = stringResource(id = R.string.recovery_phrase_disclaimer),
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
				placeholder = {
					Text(stringResource(R.string.mnemonic_example), style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
				},
				value = mnemonic,
				onValueChange = {
					if (success.value == false) viewModel.resetSuccess()
					mnemonic = it
				},
				keyboardActions = KeyboardActions(onDone = {
					keyboardController?.hide()
					onSubmit()
				}),
				modifier = Modifier
					.width(358.dp.scaledWidth())
					.height(212.dp.scaledHeight()),
				supportingText = {
					if (success.value == false) {
						Text(
							modifier = Modifier.fillMaxWidth(),
							text = stringResource(R.string.invalid_recovery_phrase),
							color = MaterialTheme.colorScheme.error,
						)
					}
				},
				isError = success.value == false,
				label = { Text(text = stringResource(id = R.string.recovery_phrase), color = MaterialTheme.colorScheme.onSurface) },
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
				Column(
					horizontalAlignment = Alignment.CenterHorizontally,
					verticalArrangement = Arrangement.spacedBy(16.dp),
				) {
					// Box(modifier = Modifier.width(286.dp.scaledWidth())) {
					MainStyledButton(
						Constants.LOGIN_TEST_TAG,
						onClick = {
							onSubmit()
						},
						content = {
							if (loading) {
								SpinningIcon(Icons.Outlined.Refresh)
							} else {
								Text(
									stringResource(id = R.string.log_in),
									style = CustomTypography.labelHuge,
								)
							}
						},
						color = MaterialTheme.colorScheme.primary,
					)

// Disable scanner for now
					// }
// 				Box(modifier = Modifier.width(56.dp.scaledWidth())) {
// 					MainStyledButton(
// 						onClick = {
// 							requestPermissionLauncher.launch(android.Manifest.permission.CAMERA)
// 						},
// 						content = {
// 							val icon = Icons.Outlined.QrCodeScanner
// 							Icon(icon, icon.name, modifier = Modifier.size(iconSize.scaledWidth()))
// 						},
// 						color = MaterialTheme.colorScheme.primary,
// 					)
// 				}
					val createAccountMessage = buildAnnotatedString {
						append(stringResource(id = R.string.new_to_nym))
						append(" ")
						pushStringAnnotation(
							tag = "create",
							annotation = appUiState.managerState.accountLinks?.signUp ?: stringResource(R.string.create_account_url),
						)
						withStyle(style = SpanStyle(color = MaterialTheme.colorScheme.primary)) {
							append(stringResource(id = R.string.create_account))
						}
						pop()
					}
					ClickableText(
						text = createAccountMessage,
						style = MaterialTheme.typography.bodyLarge.copy(
							color = MaterialTheme.colorScheme.onBackground,
							textAlign = TextAlign.Center,
						),
						modifier = Modifier.padding(bottom = 24.dp.scaledHeight()),
					) {
						createAccountMessage.getStringAnnotations(tag = "create", it, it).firstOrNull()?.let { annotation ->
							context.openWebUrl(annotation.item)
						}
					}
				}
			}
		}
	}
}
