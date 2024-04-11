package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ShapeDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.util.scaledHeight

@Composable
fun MainStyledButton(
	testTag: String? = null,
	onClick: () -> Unit,
	content: @Composable () -> Unit,
	color: Color = MaterialTheme.colorScheme.primary,
) {
	Button(
		onClick = { onClick() },
		colors =
		ButtonDefaults.buttonColors(
			containerColor = color,
		),
		modifier =
		Modifier
			.height(56.dp.scaledHeight())
			.fillMaxWidth().testTag(testTag ?: ""),
		shape =
		ShapeDefaults.Small,
	) {
		content()
	}
}
