package net.nymtech.nymvpn.ui.screens.settings.login.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.TextLinkStyles
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.animations.SpinningIcon
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.screens.settings.login.LoginViewModel
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun LoginInputSection(
	appUiState: AppUiState,
	viewModel: LoginViewModel,
	success: Boolean?,
	loading: Boolean,
	onLoadingChange: (Boolean) -> Unit,
	onRequestCameraPermission: () -> Unit,
) {
	val context = LocalContext.current
	val keyboardController = LocalSoftwareKeyboardController.current
	var mnemonic by remember { mutableStateOf("") }

	val onSubmit = {
		keyboardController?.hide()
		onLoadingChange(true)
		viewModel.onMnemonicImport(mnemonic)
	}

	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(32.dp.scaledHeight(), Alignment.Top),
	) {
		CustomTextField(
			placeholder = {
				Column {
					Text(
						stringResource(R.string.access_code),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onSurfaceVariant,
					)
					Text("")
					Text(
						stringResource(R.string.mnemonic_example),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onSurfaceVariant,
					)
				}
			},
			value = mnemonic,
			onValueChange = {
				if (success == false) viewModel.resetSuccess()
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
				if (success == false) {
					Text(
						modifier = Modifier.fillMaxWidth(),
						text = stringResource(R.string.invalid_recovery_phrase),
						color = MaterialTheme.colorScheme.error,
					)
				}
			},
			isError = success == false,
			label = {
				Text(
					text = stringResource(R.string.recovery_phrase),
					color = MaterialTheme.colorScheme.onSurface,
				)
			},
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
				MainStyledButton(
					testTag = Constants.LOGIN_TEST_TAG,
					onClick = { onSubmit() },
					content = {
						if (loading && success == null) {
							SpinningIcon(Icons.Outlined.Refresh, stringResource(R.string.refresh))
						} else {
							Text(
								stringResource(R.string.log_in),
								style = CustomTypography.labelHuge,
							)
						}
					},
					color = MaterialTheme.colorScheme.primary,
					modifier = Modifier.fillMaxWidth().height(56.dp.scaledHeight()),
				)
				// Scanner button (disabled for now)
				/*
				MainStyledButton(
					onClick = onRequestCameraPermission,
					content = {
						Icon(
							Icons.Outlined.QrCodeScanner,
							"QR Scanner",
							modifier = Modifier.size(iconSize.scaledWidth())
						)
					},
					color = MaterialTheme.colorScheme.primary,
					modifier = Modifier.width(56.dp.scaledWidth())
				)
				 */
				Text(
					text = buildAnnotatedString {
						append(stringResource(R.string.new_to_nym))
						append(" ")
						withLink(
							LinkAnnotation.Clickable(
								tag = "signUpLink",
								styles = TextLinkStyles(SpanStyle(color = MaterialTheme.colorScheme.primary)),
							) {
								val url = if (BuildConfig.FLAVOR == Constants.FDROID) {
									context.getString(R.string.pricing_url)
								} else {
									appUiState.managerState.accountLinks?.signUp ?: context.getString(R.string.create_account_url)
								}
								context.openWebUrl(url)
							},
						) {
							append(stringResource(R.string.get_access_code))
						}
					},
					style = MaterialTheme.typography.bodyLarge.copy(
						color = MaterialTheme.colorScheme.onBackground,
						textAlign = TextAlign.Center,
					),
					modifier = Modifier.padding(24.dp.scaledHeight()),
				)
			}
		}
	}
}
