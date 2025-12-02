package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ShapeDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp

@Composable
fun TransparentButton(testTag: String? = null, onClick: () -> Unit, content: @Composable () -> Unit, modifier: Modifier = Modifier, borderStroke: BorderStroke? = null) {
	Button(
		onClick = { onClick() },
		colors = ButtonDefaults.buttonColors(
			containerColor = Color.Transparent,
			contentColor = Color.White,
		),
		contentPadding = PaddingValues(),
		modifier = modifier
			.testTag(testTag ?: "")
			.defaultMinSize(1.dp, 1.dp),
		shape = ShapeDefaults.Small,
		border = borderStroke,
	) {
		content()
	}
}
