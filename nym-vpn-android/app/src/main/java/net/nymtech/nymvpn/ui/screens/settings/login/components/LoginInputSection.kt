package net.nymtech.nymvpn.ui.screens.settings.login.components

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.verticalScroll
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
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.animations.SpinningIcon
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.screens.settings.login.LoginUiState
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun LoginInputSection(
	onCreateAccountClick: () -> Unit,
	uiState: LoginUiState,
	loading: Boolean,
	mnemonic: String,
	onMnemonicChange: (String) -> Unit,
	onSubmitMnemonic: (String) -> Unit,
	onDismissError: () -> Unit,
) {
	val keyboardController = LocalSoftwareKeyboardController.current

	val submit = {
		keyboardController?.hide()
		onSubmitMnemonic(mnemonic)
	}
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
	) {
		CustomTextField(
			placeholder = {
				Column {
					Text(
						stringResource(R.string.mnemonic_example),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.outline,
					)
				}
			},
			value = mnemonic,
			onValueChange = { newValue ->
				if (uiState.success == false) onDismissError()
				onMnemonicChange(newValue)
			},
			keyboardActions = KeyboardActions(
				onDone = { submit() },
			),
			modifier = Modifier
				.fillMaxWidth()
				.height(130.dp.scaledHeight()),
			supportingText = {
				if (uiState.success == false) {
					Text(
						modifier = Modifier.fillMaxWidth(),
						text = stringResource(R.string.invalid_recovery_phrase),
						color = MaterialTheme.colorScheme.error,
					)
				}
			},
			isError = uiState.success == false,
			label = {
				Text(
					text = stringResource(R.string.recovery_phrase),
					color = MaterialTheme.colorScheme.onBackground,
				)
			},
			textStyle = MaterialTheme.typography.bodyMedium.copy(
				color = MaterialTheme.colorScheme.onSurface,
			),
		)

		Spacer(modifier = Modifier.height(14.dp.scaledHeight()))

		MainStyledButton(
			testTag = Constants.LOGIN_TEST_TAG,
			onClick = { submit() },
			content = {
				if (loading && uiState.success == null) {
					SpinningIcon(Icons.Outlined.Refresh, stringResource(R.string.refresh))
				} else {
					Text(
						stringResource(R.string.log_in),
						style = CustomTypography.buttonMain,
					)
				}
			},
			color = MaterialTheme.colorScheme.primary,
			modifier = Modifier
				.fillMaxWidth()
				.height(56.dp.scaledHeight()),
		)
		Spacer(modifier = Modifier.height(24.dp.scaledHeight()))

		Row(
			modifier = Modifier.fillMaxWidth(),
			horizontalArrangement = Arrangement.Center,
			verticalAlignment = Alignment.CenterVertically,
		) {
			Text(
				text = stringResource(R.string.new_to_nym),
				style = MaterialTheme.typography.labelLarge,
				color = MaterialTheme.colorScheme.outline,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
			Spacer(modifier = Modifier.width(4.dp))
			Text(
				text = stringResource(R.string.onboarding_create_account_button),
				style = MaterialTheme.typography.labelLarge,
				color = MaterialTheme.colorScheme.primary,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				modifier = Modifier.clickable { onCreateAccountClick() },
			)
		}
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, showBackground = true)
@Composable
internal fun PreviewLoginInputSection_Default() {
	NymVPNTheme(Theme.default()) {
		var mnemonic by remember { mutableStateOf("") }

		Column(
			modifier = Modifier
				.fillMaxSize()
				.background(MaterialTheme.colorScheme.background)
				.verticalScroll(rememberScrollState())
				.padding(24.dp),
		) {
			LoginInputSection(
				onCreateAccountClick = {},
				uiState = LoginUiState(),
				loading = false,
				mnemonic = "",
				onMnemonicChange = { mnemonic = it },
				onSubmitMnemonic = {},
				onDismissError = {},
			)
		}
	}
}
