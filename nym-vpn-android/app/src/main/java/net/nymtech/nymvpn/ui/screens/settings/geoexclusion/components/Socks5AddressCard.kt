package net.nymtech.nymvpn.ui.screens.settings.geoexclusion.components

import androidx.annotation.StringRes
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField

private const val LOOPBACK_ADDRESS = "127.0.0.1"

@Composable
fun Socks5AddressCard(onCopyServer: () -> Unit, proxyAddress: String, onCopy: () -> Unit, portInput: String, @StringRes portError: Int?, onPortChange: (String) -> Unit, onPortCommit: () -> Unit) {
	Card(
		shape = RoundedCornerShape(14.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
		modifier = Modifier
			.fillMaxWidth()
			.wrapContentHeight(),
	) {
		AddressRow(
			label = stringResource(R.string.geo_exclusion_server_title),
			value = LOOPBACK_ADDRESS,
			onCopy = onCopyServer,
		)
		HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant, thickness = 1.dp)
		AddressRow(
			label = stringResource(R.string.geo_exclusion_sock55_title),
			value = proxyAddress,
			onCopy = onCopy,
		)
		HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant, thickness = 1.dp)
		Column(modifier = Modifier.padding(16.dp)) {
			Text(
				text = stringResource(R.string.geo_exclusion_custom_port_title),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.padding(bottom = 6.dp),
			)
			CustomTextField(
				value = portInput,
				label = {},
				placeholder = { Text(stringResource(R.string.geo_exclusion_ports_text)) },
				isError = portError != null,
				supportingText = portError?.let { res -> { Text(stringResource(res)) } },
				singleLine = true,
				keyboardOptions = KeyboardOptions(
					keyboardType = KeyboardType.Number,
					imeAction = ImeAction.Done,
				),
				textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onPrimaryContainer),
				keyboardActions = KeyboardActions(onDone = { onPortCommit() }),
				onValueChange = onPortChange,
				modifier = Modifier.fillMaxWidth(),
			)
			Text(
				text = stringResource(R.string.geo_exclusion_ports_text),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onBackground.copy(alpha = 0.5f),
				modifier = Modifier.padding(top = 6.dp),
			)
		}
	}
}

@Composable
private fun AddressRow(label: String, value: String, onCopy: () -> Unit) {
	Column(modifier = Modifier.padding(16.dp)) {
		Row(
			modifier = Modifier.fillMaxWidth(),
			verticalAlignment = Alignment.CenterVertically,
		) {
			Text(
				text = label,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.weight(1f),
			)
			Text(
				text = value,
				style = MaterialTheme.typography.bodyLarge,
				color = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.clickable { onCopy() },
			)
			Spacer(Modifier.width(8.dp))
			Icon(
				imageVector = Icons.Outlined.ContentCopy,
				contentDescription = stringResource(R.string.geo_exclusion_copy_proxy_label),
				modifier = Modifier
					.size(16.dp)
					.clickable { onCopy() },
				tint = MaterialTheme.colorScheme.onBackground,
			)
		}
	}
}
