package net.nymtech.nymvpn.ui.screens.scan.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.screens.scan.RedeemCodeUiState
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun RedeemCodeForm(uiState: RedeemCodeUiState, onCodeChange: (String) -> Unit, onSubmit: () -> Unit, modifier: Modifier = Modifier) {
	val keyboardController = LocalSoftwareKeyboardController.current
	val checkingDescription = stringResource(R.string.redeem_code_checking)
	val canSubmit = !uiState.loading && uiState.code.isNotBlank()
	val submit = {
		if (canSubmit) {
			keyboardController?.hide()
			onSubmit()
		}
	}

	Column(
		modifier = modifier.fillMaxWidth(),
		verticalArrangement = Arrangement.spacedBy(16.dp),
	) {
		Text(
			text = stringResource(R.string.redeem_code_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.outline,
		)
		CustomTextField(
			value = uiState.code,
			onValueChange = onCodeChange,
			singleLine = true,
			enabled = !uiState.loading,
			isError = uiState.errorRes != null,
			label = {
				Text(
					text = stringResource(R.string.redeem_code_label),
					color = MaterialTheme.colorScheme.onBackground,
				)
			},
			placeholder = {
				Text(
					text = stringResource(R.string.redeem_code_placeholder),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.outline,
				)
			},
			supportingText = uiState.errorRes?.let { errorRes ->
				{
					Text(
						text = stringResource(errorRes),
						color = MaterialTheme.colorScheme.error,
						modifier = Modifier
							.fillMaxWidth()
							.semantics { liveRegion = LiveRegionMode.Polite },
					)
				}
			},
			keyboardOptions = KeyboardOptions(
				keyboardType = KeyboardType.Ascii,
				capitalization = KeyboardCapitalization.None,
				autoCorrectEnabled = false,
				imeAction = ImeAction.Done,
			),
			keyboardActions = KeyboardActions(onDone = { submit() }),
			modifier = Modifier.fillMaxWidth(),
		)
		MainStyledButton(
			onClick = submit,
			enabled = canSubmit,
			content = {
				if (uiState.loading) {
					CircularProgressIndicator(
						color = MaterialTheme.colorScheme.onPrimary,
						strokeWidth = 2.dp,
						modifier = Modifier
							.size(20.dp)
							.semantics {
								contentDescription = checkingDescription
								liveRegion = LiveRegionMode.Polite
							},
					)
				} else {
					Text(stringResource(R.string.redeem_code_submit), style = MaterialTheme.typography.titleMedium)
				}
			},
			modifier = Modifier
				.fillMaxWidth()
				.height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
		)
	}
}
