package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun ScaledSwitch(checked: Boolean, modifier: Modifier = Modifier, onClick: (checked: Boolean) -> Unit, enabled: Boolean = true) {
	Switch(
		checked,
		{ onClick(it) },
		modifier =
		modifier.padding(0.dp),
		enabled = enabled,
	)
}
