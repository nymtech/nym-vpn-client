package net.nymtech.nymvpn.ui.screens.settings.dns.components

import android.content.res.Configuration
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.ShapeDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.isValidIPv4
import net.nymtech.nymvpn.util.extensions.isValidIPv6
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun CustomDnsCard(
	initialDns: List<String>,
	dnsEnabled: Boolean,
	connectedForUi: Boolean,
	onDnsEnable: (enabled: Boolean) -> Unit,
	modifier: Modifier = Modifier,
	onSave: (List<String>) -> Unit,
	onDnsListChange: (List<String>) -> Unit,
	toggleEnabled: Boolean,
) {
	val dnsItems = remember { mutableStateListOf<DnsEntry>() }
	var nextItemId by remember { mutableLongStateOf(1L) }
	val validateList = remember { listOf("0.0.0.0", "255.255.255.255") }

	LaunchedEffect(initialDns) {
		dnsItems.clear()
		nextItemId = 1L
		initialDns.forEach { value ->
			dnsItems.add(DnsEntry(nextItemId++, value))
		}
	}

	var dnsInput by rememberSaveable { mutableStateOf("") }
	val normalizedDnsInput = dnsInput.trim()

	val isDnsInputValid = remember(normalizedDnsInput) {
		normalizedDnsInput.isNotEmpty() &&
			normalizedDnsInput !in validateList &&
			(isValidIPv4(normalizedDnsInput) || isValidIPv6(normalizedDnsInput))
	}

	val shouldShowInputError = remember(normalizedDnsInput, isDnsInputValid) {
		normalizedDnsInput.isNotEmpty() && !isDnsInputValid
	}

	val currentDnsValues by remember {
		derivedStateOf { dnsItems.map { it.value } }
	}

	LaunchedEffect(currentDnsValues) {
		onDnsListChange(currentDnsValues)
	}

	val hasUnsavedChanges by remember(initialDns, currentDnsValues) {
		derivedStateOf { currentDnsValues != initialDns }
	}

	val canAddDns by remember(isDnsInputValid, dnsItems.size) {
		derivedStateOf { isDnsInputValid && dnsItems.size < 5 }
	}

	val saveTextRes =
		if (connectedForUi && dnsEnabled) {
			R.string.dns_custom_button_save_reconnect
		} else {
			R.string.dns_custom_button_save
		}

	Card(
		modifier = modifier.fillMaxWidth(),
		shape = RoundedCornerShape(8.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
	) {
		Column(modifier = Modifier.padding(16.dp)) {
			Row(
				modifier = Modifier.fillMaxWidth(),
				verticalAlignment = Alignment.CenterVertically,
			) {
				Text(
					text = stringResource(R.string.dns_custom_title),
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier.weight(1f),
				)

				ScaledSwitch(
					checked = dnsEnabled,
					onClick = { if (toggleEnabled) onDnsEnable.invoke(it) },
					enabled = toggleEnabled,
				)
			}

			Text(
				text = stringResource(R.string.dns_custom_description),
				style = Typography.bodySmall,
				color = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.padding(top = 16.dp),
			)

			if (dnsItems.isNotEmpty()) {
				Spacer(Modifier.height(16.dp))

				Text(
					text = stringResource(R.string.dns_custom_list_entries, dnsItems.size),
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)

				Spacer(Modifier.height(12.dp))
				HorizontalDivider(color = MaterialTheme.colorScheme.outline.copy(alpha = 0.60f))

				DnsReorderableList(
					dns = dnsItems,
					onMove = { from, to -> dnsItems.move(from, to) },
					onDelete = { index -> dnsItems.removeAt(index) },
				)
			}

			if (dnsItems.size < 5) {
				Spacer(Modifier.height(16.dp))

				Row(
					modifier = Modifier.fillMaxWidth(),
					verticalAlignment = Alignment.CenterVertically,
				) {
					OutlinedTextField(
						value = dnsInput,
						onValueChange = { dnsInput = it },
						modifier = Modifier
							.weight(1f)
							.heightIn(min = 56.dp)
							.padding(end = 16.dp),
						singleLine = true,
						label = {
							Text(
								text = stringResource(R.string.dns_custom_list_title),
								style = MaterialTheme.typography.bodyMedium,
							)
						},
						placeholder = {
							Text(
								text = stringResource(R.string.dns_custom_list_hint),
								style = MaterialTheme.typography.bodyMedium,
							)
						},
						isError = shouldShowInputError,
						keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Ascii),
						shape = ShapeDefaults.Small,
					)

					MainStyledButton(
						onClick = {
							dnsItems.add(DnsEntry(nextItemId++, normalizedDnsInput))
							dnsInput = ""
						},
						enabled = canAddDns,
						content = {
							Text(
								text = stringResource(R.string.button_add),
								style = MaterialTheme.typography.titleMedium,
								maxLines = 1,
								softWrap = false,
								overflow = TextOverflow.Ellipsis,
								modifier = Modifier.padding(horizontal = 8.dp),
							)
						},
						modifier = Modifier
							.padding(top = 6.dp)
							.heightIn(52.dp)
							.widthIn(min = 62.dp.scaledWidth()),
					)
				}

				if (shouldShowInputError) {
					Spacer(Modifier.height(8.dp))
					Text(
						text = stringResource(R.string.dns_custom_list_error),
						color = MaterialTheme.colorScheme.error,
						style = MaterialTheme.typography.bodyMedium,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				}
			}

			Spacer(Modifier.height(18.dp))

			MainStyledButton(
				onClick = {
					val snapshot = currentDnsValues
					onSave(snapshot)
				},
				enabled = hasUnsavedChanges,
				content = {
					Text(
						stringResource(saveTextRes),
						style = MaterialTheme.typography.titleMedium,
					)
				},
				modifier = Modifier.fillMaxWidth().height(48.dp.scaledHeight()),
				shape = RoundedCornerShape(12.dp),
			)
		}
	}
}

data class DnsEntry(val id: Long, val value: String)

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun CustomDnsCardPreview() {
	NymVPNTheme(Theme.default()) {
		CustomDnsCard(
			initialDns = listOf(
				"192.168.1.1",
				"10.0.0.1",
				"208.67.222.222",
			),
			dnsEnabled = false,
			connectedForUi = true,
			onDnsEnable = {},
			modifier = Modifier.padding(16.dp),
			onSave = {},
			onDnsListChange = {},
			toggleEnabled = true,
		)
	}
}
