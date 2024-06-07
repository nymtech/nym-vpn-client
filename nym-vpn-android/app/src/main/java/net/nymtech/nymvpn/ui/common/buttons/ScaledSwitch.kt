package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.util.scaledHeight

@Composable
fun ScaledSwitch(checked: Boolean, modifier: Modifier = Modifier, onClick: (checked: Boolean) -> Unit, enabled: Boolean = true) {
	Switch(
		checked,
		{ onClick(it) },
		modifier =
		modifier.scale(1.2.dp.scaledHeight().value),
		enabled = enabled,
	)
}
