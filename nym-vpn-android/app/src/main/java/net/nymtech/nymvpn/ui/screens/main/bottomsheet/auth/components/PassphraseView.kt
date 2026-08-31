package net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.animations.SpinningIcon
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun PassphraseView(
	onBackClick: () -> Unit,
	mnemonicError: MnemonicError?,
	loading: Boolean,
	mnemonic: String,
	onMnemonicChange: (String) -> Unit,
	onSubmitMnemonic: () -> Unit,
	modifier: Modifier = Modifier,
) {
	val keyboardController = LocalSoftwareKeyboardController.current
	val isError = mnemonicError != null

	val submit = {
		if (!loading && mnemonic.isNotBlank()) {
			keyboardController?.hide()
			onSubmitMnemonic()
		}
	}

	Column(
		modifier = modifier
			.background(MaterialTheme.colorScheme.surface)
			.fillMaxWidth()
			.padding(horizontal = 18.dp)
			.padding(top = 16.dp, bottom = 41.dp),
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(22.dp),
	) {
		Box(modifier = Modifier.fillMaxWidth()) {
			IconButton(
				onClick = onBackClick,
				modifier = Modifier
					.align(Alignment.CenterStart)
					.size(24.dp),
			) {
				Icon(
					imageVector = Icons.AutoMirrored.Filled.KeyboardArrowLeft,
					tint = MaterialTheme.colorScheme.onBackground,
					contentDescription = stringResource(R.string.button_back),
				)
			}
			Icon(
				imageVector = ImageVector.vectorResource(R.drawable.app_label),
				contentDescription = stringResource(R.string.app_name),
				tint = MaterialTheme.colorScheme.onPrimaryContainer,
				modifier = Modifier.align(Alignment.Center),
			)
		}

		Text(
			text = stringResource(R.string.auth_log_in_title),
			style = MaterialTheme.typography.headlineSmall,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
		)

		CustomTextField(
			placeholder = {
				Text(
					stringResource(R.string.mnemonic_example),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.outline,
				)
			},
			value = mnemonic,
			onValueChange = onMnemonicChange,
			keyboardActions = KeyboardActions(onDone = { submit() }),
			keyboardOptions = KeyboardOptions(
				keyboardType = KeyboardType.Password,
				autoCorrectEnabled = false,
				imeAction = ImeAction.Done,
			),
			modifier = Modifier
				.fillMaxWidth()
				.height(148.dp.scaledHeight()),
			supportingText = {
				if (isError) {
					Text(
						modifier = Modifier.fillMaxWidth(),
						text = when (mnemonicError) {
							MnemonicError.INVALID_RECOVERY_PHRASE -> stringResource(R.string.invalid_recovery_phrase)
							null -> ""
						},
						color = MaterialTheme.colorScheme.error,
					)
				}
			},
			isError = isError,
			label = {
				Text(
					text = stringResource(R.string.auth_passphrase_input_placeholder),
					color = MaterialTheme.colorScheme.onBackground,
				)
			},
			textStyle = MaterialTheme.typography.bodyMedium.copy(
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			),
		)

		MainStyledButton(
			onClick = submit,
			enabled = !loading && mnemonic.isNotBlank(),
			content = {
				if (loading) {
					SpinningIcon(Icons.Outlined.Lock, "")
				} else {
					Text(
						stringResource(R.string.auth_passphrase_login_button),
						style = MaterialTheme.typography.titleMedium,
					)
				}
			},
			modifier = Modifier
				.fillMaxWidth()
				.height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
		)
	}
}

enum class MnemonicError {
	INVALID_RECOVERY_PHRASE,
}

@Preview(name = "PassphraseViewPreview", uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun PreviewPassphraseViewDark() {
	NymVPNTheme(Theme.DARK_MODE) {
		PassphraseView(
			onBackClick = {},
			mnemonicError = null,
			loading = false,
			mnemonic = "",
			onMnemonicChange = {},
			onSubmitMnemonic = {},
		)
	}
}
