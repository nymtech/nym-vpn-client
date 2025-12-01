package net.nymtech.nymvpn.ui.common.buttons

import android.content.res.Configuration
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun ScaledSwitch(checked: Boolean, onClick: (checked: Boolean) -> Unit, enabled: Boolean = true) {
	Switch(
		checked,
		{ onClick(it) },
		Modifier.scale((52.dp.scaledHeight() / 52.dp)),
		colors = SwitchDefaults.colors(
			checkedBorderColor = MaterialTheme.colorScheme.primary,
			checkedThumbColor = MaterialTheme.colorScheme.surface,
			checkedTrackColor = MaterialTheme.colorScheme.primary,
			uncheckedBorderColor = MaterialTheme.colorScheme.outline,
			uncheckedThumbColor = MaterialTheme.colorScheme.outline,
		)
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewScaledSwitch() {
	NymVPNTheme(Theme.default()) {
		ScaledSwitch(
			true, {}
		)
	}
}
