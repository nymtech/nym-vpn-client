package net.nymtech.nymvpn.ui.common.textbox

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.theme.CustomColors

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CustomTextField(
	value: String,
	textStyle: TextStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onSurface),
	label: @Composable (() -> Unit),
	onValueChange: (value: String) -> Unit = {},
	modifier: Modifier = Modifier,
	singleLine: Boolean = false,
	placeholder: @Composable (() -> Unit)? = null,
	supportingText: @Composable (() -> Unit)? = null,
	leading: @Composable (() -> Unit)? = null,
	trailing: @Composable (() -> Unit)? = null,
	isError: Boolean = false,
	readOnly: Boolean = false,
	enabled: Boolean = true,
) {
	val interactionSource = remember { MutableInteractionSource() }
	val space = " "
	BasicTextField(
		value = value,
		textStyle = textStyle,
		onValueChange = {
			onValueChange(it)
		},
		readOnly = readOnly,
		cursorBrush = SolidColor(MaterialTheme.colorScheme.onSurface),
		modifier = modifier,
		interactionSource = interactionSource,
		enabled = enabled,
		singleLine = singleLine,
	) {
		OutlinedTextFieldDefaults.DecorationBox(
			value = space + value,
			innerTextField = {
				if (value.isEmpty()) {
					if (placeholder != null) {
						placeholder()
					}
				}
				it.invoke()
			},
			leadingIcon = leading,
			trailingIcon = trailing,
			singleLine = singleLine,
			supportingText = supportingText,
			colors = TextFieldDefaults.colors().copy(
				disabledLabelColor = MaterialTheme.colorScheme.onSurface,
				disabledContainerColor = MaterialTheme.colorScheme.background,
				focusedIndicatorColor = CustomColors.outlineVariant,
				disabledIndicatorColor = CustomColors.outlineVariant,
				unfocusedIndicatorColor = CustomColors.outlineVariant,
				focusedLabelColor = MaterialTheme.colorScheme.onSurface,
				focusedContainerColor = MaterialTheme.colorScheme.background,
				unfocusedContainerColor = MaterialTheme.colorScheme.background,
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
						disabledLabelColor = MaterialTheme.colorScheme.onSurface,
						disabledContainerColor = MaterialTheme.colorScheme.background,
						focusedIndicatorColor = CustomColors.outlineVariant,
						disabledIndicatorColor = CustomColors.outlineVariant,
						unfocusedIndicatorColor = CustomColors.outlineVariant,
						focusedLabelColor = MaterialTheme.colorScheme.onSurface,
						focusedContainerColor = MaterialTheme.colorScheme.background,
						unfocusedContainerColor = MaterialTheme.colorScheme.background,
						focusedTextColor = MaterialTheme.colorScheme.onSurface,
						cursorColor = MaterialTheme.colorScheme.onSurface,
					),
					shape = RoundedCornerShape(8.dp),
					focusedBorderThickness = 0.5.dp,
					unfocusedBorderThickness = 0.5.dp,
				)
			},
		)
	}
}
