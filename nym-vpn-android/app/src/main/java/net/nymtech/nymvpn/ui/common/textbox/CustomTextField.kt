package net.nymtech.nymvpn.ui.common.textbox

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CustomTextField(
	value: String,
	modifier: Modifier = Modifier,
	textStyle: TextStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onSurface),
	label: @Composable (() -> Unit),
	onValueChange: (value: String) -> Unit = {},
	singleLine: Boolean = false,
	placeholder: @Composable (() -> Unit)? = null,
	supportingText: @Composable (() -> Unit)? = null,
	keyboardActions: KeyboardActions = KeyboardActions(),
	keyboardOptions: KeyboardOptions = KeyboardOptions.Default.copy(imeAction = ImeAction.Done),
	leading: @Composable (() -> Unit)? = null,
	trailing: @Composable (() -> Unit)? = null,
	showClearIcon: Boolean = false,
	isError: Boolean = false,
	readOnly: Boolean = false,
	enabled: Boolean = true,
	subtitle: @Composable (() -> Unit)? = null,
	containerColor: Color = MaterialTheme.colorScheme.surfaceContainerLow,
) {
	val interactionSource = remember { MutableInteractionSource() }
	val isFocused by interactionSource.collectIsFocusedAsState()

	val space = " "

	val trailingIcon: @Composable (() -> Unit)? =
		if (showClearIcon && value.isNotEmpty() && enabled && !readOnly) {
			{
				IconButton(
					onClick = { onValueChange("") },
					modifier = Modifier
						.size(32.dp)
						.semantics { contentDescription = "Clear text" },
				) {
					Icon(
						imageVector = Icons.Filled.Close,
						contentDescription = null,
						tint = MaterialTheme.colorScheme.onSurface,
						modifier = Modifier.size(20.dp),
					)
				}
			}
		} else {
			trailing
		}

	BasicTextField(
		value = value,
		textStyle = textStyle,
		onValueChange = { onValueChange(it) },
		readOnly = readOnly,
		keyboardActions = keyboardActions,
		keyboardOptions = keyboardOptions,
		cursorBrush = SolidColor(MaterialTheme.colorScheme.onSurface),
		modifier = modifier,
		interactionSource = interactionSource,
		enabled = enabled,
		singleLine = singleLine,
	) {
		OutlinedTextFieldDefaults.DecorationBox(
			value = space + value,
			innerTextField = {
				Column {
					if (value.isEmpty() && !isFocused) {
						placeholder?.invoke()
					} else {
						it.invoke()
					}
					subtitle?.invoke()
				}
			},
			contentPadding = OutlinedTextFieldDefaults.contentPadding(top = 12.dp, bottom = 12.dp),
			leadingIcon = leading,
			trailingIcon = trailingIcon,
			singleLine = singleLine,
			supportingText = supportingText,
			colors = TextFieldDefaults.colors().copy(
				disabledLabelColor = MaterialTheme.colorScheme.onSurface,
				disabledContainerColor = containerColor,
				focusedIndicatorColor = MaterialTheme.colorScheme.onSurface,
				disabledIndicatorColor = MaterialTheme.colorScheme.outline,
				unfocusedIndicatorColor = MaterialTheme.colorScheme.outline,
				focusedLabelColor = MaterialTheme.colorScheme.onSurface,
				focusedContainerColor = containerColor,
				unfocusedContainerColor = containerColor,
				focusedTextColor = MaterialTheme.colorScheme.onSurface,
				cursorColor = MaterialTheme.colorScheme.onSurface,
			),
			enabled = enabled,
			label = label,
			visualTransformation = VisualTransformation.None,
			interactionSource = interactionSource,
			placeholder = placeholder,
			container = {
				OutlinedTextFieldDefaults.ContainerBox(
					enabled,
					isError = isError,
					interactionSource,
					colors = TextFieldDefaults.colors().copy(
						errorContainerColor = containerColor,
						disabledLabelColor = MaterialTheme.colorScheme.onSurface,
						disabledContainerColor = containerColor,
						focusedIndicatorColor = MaterialTheme.colorScheme.onSurface,
						disabledIndicatorColor = MaterialTheme.colorScheme.outline,
						unfocusedIndicatorColor = MaterialTheme.colorScheme.outline,
						focusedLabelColor = MaterialTheme.colorScheme.onSurface,
						focusedContainerColor = containerColor,
						unfocusedContainerColor = containerColor,
						focusedTextColor = MaterialTheme.colorScheme.onSurface,
						cursorColor = MaterialTheme.colorScheme.onSurface,
					),
					shape = RoundedCornerShape(12.dp),
					focusedBorderThickness = 0.5.dp,
					unfocusedBorderThickness = 0.5.dp,
				)
			},
		)
	}
}
