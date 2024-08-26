package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.ShapeDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun MainStyledButtonOutlined(
	testTag: String? = null,
	onClick: () -> Unit,
	content: @Composable () -> Unit,
	color: Color = MaterialTheme.colorScheme.primary,
) {
	OutlinedButton(
		onClick = { onClick() },
		contentPadding = PaddingValues(),
		border = BorderStroke(1.dp, color),
		modifier =
		Modifier
			.height(56.dp.scaledHeight())
			.fillMaxWidth().testTag(testTag ?: "").defaultMinSize(1.dp, 1.dp),
		shape =
		ShapeDefaults.Small,
	) {
		content()
	}
}
