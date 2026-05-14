package net.nymtech.nymvpn.ui.common.buttons

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun ScaledSwitch(checked: Boolean, onClick: (checked: Boolean) -> Unit, enabled: Boolean = true) {
	val primary = MaterialTheme.colorScheme.primary
	val onSurfaceVariant = LocalNymColors.current.switchBackground

	Switch(
		checked = checked,
		onCheckedChange = onClick,
		enabled = enabled,
		modifier = Modifier.size(width = 46.dp, height = 24.dp),
		colors = SwitchDefaults.colors(
			checkedTrackColor = primary,
			checkedThumbColor = Color.White,
			checkedBorderColor = primary,

			uncheckedTrackColor = onSurfaceVariant,
			uncheckedThumbColor = Color.White,
			uncheckedBorderColor = onSurfaceVariant,

			disabledCheckedTrackColor = primary.copy(alpha = 0.3f),
			disabledCheckedThumbColor = MaterialTheme.colorScheme.surface,
			disabledCheckedBorderColor = primary.copy(alpha = 0.3f),

			disabledUncheckedTrackColor = onSurfaceVariant.copy(alpha = 0.4f),
			disabledUncheckedThumbColor = Color.White.copy(alpha = 0.5f),
			disabledUncheckedBorderColor = onSurfaceVariant.copy(alpha = 0.4f),
		),
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
private fun PreviewScaledSwitchOff() {
	NymVPNTheme(Theme.default()) {
		Column(
			verticalArrangement = Arrangement.spacedBy(8.dp),
			modifier = Modifier.padding(16.dp),
		) {
			ScaledSwitch(checked = false, onClick = {})
			ScaledSwitch(checked = true, onClick = {})
			ScaledSwitch(checked = false, onClick = {}, enabled = false)
			ScaledSwitch(checked = true, onClick = {}, enabled = false)
		}
	}
}
