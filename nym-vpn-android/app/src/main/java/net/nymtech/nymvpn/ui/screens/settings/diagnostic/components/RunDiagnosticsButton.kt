package net.nymtech.nymvpn.ui.screens.settings.diagnostic.components

import androidx.annotation.StringRes
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.util.extensions.scaledHeight

/**
 * The green diagnostics button: "Run diagnostics" on Data, privacy & logs opens the
 * diagnostic screen, which starts the run itself; "Re-run diagnostics" on that screen runs it again.
 */
@Composable
fun RunDiagnosticsButton(onClick: () -> Unit, @StringRes labelRes: Int = R.string.diagnostic_run_button) {
	MainStyledButton(
		onClick = onClick,
		content = {
			Text(
				text = stringResource(labelRes),
				style = MaterialTheme.typography.titleMedium,
			)
		},
		modifier = Modifier
			.fillMaxWidth()
			.height(48.dp.scaledHeight()),
		shape = RoundedCornerShape(12.dp),
	)
}
