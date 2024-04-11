package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.ShapeDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.scaledHeight

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ListOptionSelectionButton(
	label: String,
	value: String,
	onClick: () -> Unit,
	leadingIcon: @Composable () -> Unit,
	trailingIcon: ImageVector = ImageVector.vectorResource(R.drawable.link_arrow_right),
) {
	val interactionSource = remember { MutableInteractionSource() }
	val colors =
		OutlinedTextFieldDefaults.colors(
			disabledTextColor = MaterialTheme.colorScheme.onSurface,
			disabledContainerColor = Color.Transparent,
			disabledBorderColor = MaterialTheme.colorScheme.outline,
			disabledLeadingIconColor = MaterialTheme.colorScheme.onSurface,
			disabledTrailingIconColor = MaterialTheme.colorScheme.onSurface,
			disabledLabelColor = MaterialTheme.colorScheme.onSurfaceVariant,
			disabledPlaceholderColor = MaterialTheme.colorScheme.onSurface,
			disabledSupportingTextColor = MaterialTheme.colorScheme.onSurface,
			unfocusedLabelColor = MaterialTheme.colorScheme.onSurface,
			disabledPrefixColor = MaterialTheme.colorScheme.onSurface,
			disabledSuffixColor = MaterialTheme.colorScheme.onSurface,
		)
	BasicTextField(
		value = value,
		readOnly = true,
		enabled = false,
		onValueChange = {},
		modifier =
		Modifier
			.fillMaxWidth()
			.height(60.dp.scaledHeight())
			.defaultMinSize(minHeight = 1.dp, minWidth = 1.dp)
			.clickable(interactionSource = interactionSource, indication = null) { onClick() },
	) {
		OutlinedTextFieldDefaults.DecorationBox(
			value = value,
			leadingIcon = {
				leadingIcon()
			},
			trailingIcon = {
				Icon(trailingIcon, trailingIcon.name, tint = MaterialTheme.colorScheme.onSurface)
			},
			label = {
				Text(
					label,
					style = MaterialTheme.typography.bodySmall,
					modifier = Modifier.padding(start = 8.dp),
				)
			},
			enabled = false,
			contentPadding = PaddingValues(1.dp),
			singleLine = true,
			placeholder = {
				Text(
					value,
					style = MaterialTheme.typography.bodyLarge,
					overflow = TextOverflow.Visible,
				)
			},
			visualTransformation = VisualTransformation.None,
			innerTextField = {
				Text(
					value,
					style = MaterialTheme.typography.bodyLarge,
					overflow = TextOverflow.Visible,
				)
			},
			interactionSource = interactionSource,
			colors = colors,
			container = {
				OutlinedTextFieldDefaults.ContainerBox(
					enabled = false,
					false,
					interactionSource,
					colors,
					focusedBorderThickness = 1.dp,
					unfocusedBorderThickness = 1.dp,
					shape = ShapeDefaults.Small,
				)
			},
		)
	}
}
