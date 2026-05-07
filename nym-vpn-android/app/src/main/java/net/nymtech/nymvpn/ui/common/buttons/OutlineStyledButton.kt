package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.border
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
fun OutlineStyledButton(
	testTag: String? = null,
	onClick: () -> Unit,
	content: @Composable () -> Unit,
	enabled: Boolean = true,
	modifier: Modifier = Modifier,
	borderColor: Color = MaterialTheme.colorScheme.onBackground,
	backgroundColor: Color = Color.Transparent,
	shape: Shape = ShapeDefaults.Small,
) {
	Button(
		onClick = { onClick() },
		colors = ButtonDefaults.buttonColors(
			containerColor = backgroundColor,
			contentColor = borderColor,
		),
		enabled = enabled,
		contentPadding = PaddingValues(),
		modifier = modifier
			.testTag(testTag ?: "")
			.defaultMinSize(1.dp, 1.dp)
			.border(
				width = 1.dp,
				color = borderColor,
				shape = shape,
			),
		shape = shape,
	) {
		content()
	}
}
