package net.nymtech.nymvpn.ui.screens.hop

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.scaledHeight

@Composable
fun GatewayModalBody() {
	Column(verticalArrangement = Arrangement.spacedBy(16.dp.scaledHeight())) {
		Text(
			text = stringResource(R.string.gateway_modal_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onSurfaceVariant,
			textAlign = TextAlign.Center,
		)
		// TODO wait for blog article to be ready
// 					CompositionLocalProvider(
// 						LocalMinimumInteractiveComponentEnforcement provides false,
// 					) {
// 						TextButton(
// 							onClick = {}
// 						) {
// 							Row(
// 								modifier = Modifier.fillMaxWidth(),
// 								horizontalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterHorizontally),
// 								verticalAlignment = Alignment.CenterVertically) {
// 								Text(stringResource(id = R.string.continue_reading), style = MaterialTheme.typography.bodyMedium)
// 								val icon = Icons.AutoMirrored.Outlined.OpenInNew
// 								Icon(icon, icon.name, Modifier.size(16.dp))
// 							}
// 						}
// 					}
	}
}
