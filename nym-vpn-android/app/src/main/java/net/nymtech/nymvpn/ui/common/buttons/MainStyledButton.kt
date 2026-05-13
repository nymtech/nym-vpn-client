package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ShapeDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp

@Composable
fun MainStyledButton(
	testTag: String? = null,
	onClick: () -> Unit,
	content: @Composable () -> Unit,
	color: Color = MaterialTheme.colorScheme.primary,
	textColor: Color = MaterialTheme.colorScheme.onPrimary,
	disabledTextColor: Color = MaterialTheme.colorScheme.secondary,
	modifier: Modifier = Modifier,
	enabled: Boolean = true,
	borderStroke: BorderStroke? = null,
	shape: Shape = ShapeDefaults.Small,
) {
	Button(
		onClick = { onClick() },
		colors =
		ButtonDefaults.buttonColors(
			containerColor = color,
			contentColor = textColor,
			disabledContentColor = disabledTextColor,
		),
		border = borderStroke,
		enabled = enabled,
		contentPadding = PaddingValues(),
		modifier =
		modifier.testTag(testTag ?: "").defaultMinSize(1.dp, 1.dp),
		shape = shape,
	) {
		content()
	}
}
