package net.nymtech.nymvpn.ui.screens.settings.dns

import android.content.res.Configuration
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.settings.dns.DnsViewModel.Companion.DEFAULT_DNS_SERVERS
import net.nymtech.nymvpn.ui.screens.settings.dns.components.CustomDnsCard
import net.nymtech.nymvpn.ui.screens.settings.dns.modal.SaveChangesModal
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.safePopBackStack
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun DnsScreen(appUiState: AppUiState, onBackEventConsume: () -> Unit, onBackClickEventTriggered: Boolean = false, viewModel: DnsViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val customDns by viewModel.customDns.collectAsState()

	var initialCustomDns by remember { mutableStateOf<List<String>?>(null) }
	LaunchedEffect(customDns) {
		if (initialCustomDns == null) initialCustomDns = customDns.toList()
	}

	var initialDnsEnabled by remember { mutableStateOf<Boolean?>(null) }
	LaunchedEffect(appUiState.settings.customDnsEnabled) {
		if (initialDnsEnabled == null) initialDnsEnabled = appUiState.settings.customDnsEnabled
	}

	val onNavigateBack = remember {
		{
			onBackEventConsume()
			navController.safePopBackStack()
		}
	}

	DnsScreen(
		defaultDns = DEFAULT_DNS_SERVERS,
		customDns = customDns,
		dnsEnabled = appUiState.settings.customDnsEnabled,
		onDnsEnable = { viewModel.onCustomDnsEnable(it) },
		onSave = { viewModel.saveDnsList(it) },
		onBackClickEventTriggered = onBackClickEventTriggered,
		onNavigateBack = onNavigateBack,
		onReconnect = { viewModel.reconnect() },
		initialDnsEnabled = initialDnsEnabled,
		initialCustomDns = initialCustomDns,
	)
}

private fun shouldReconnect(initialEnabled: Boolean?, initialList: List<String>?, currentEnabled: Boolean, currentList: List<String>): Boolean {
	if (initialEnabled == null || initialList == null) return false

	val toggleChanged = currentEnabled != initialEnabled
	val listChangedWhileEnabled = currentEnabled && (currentList != initialList)
	return toggleChanged || listChangedWhileEnabled
}

@Composable
private fun DnsScreen(
	defaultDns: List<String>,
	customDns: List<String>,
	dnsEnabled: Boolean,
	onDnsEnable: (enabled: Boolean) -> Unit,
	onSave: (List<String>) -> Unit,
	onBackClickEventTriggered: Boolean,
	onNavigateBack: () -> Unit,
	onReconnect: () -> Unit,
	initialDnsEnabled: Boolean?,
	initialCustomDns: List<String>?,
) {
	val scrollState = rememberScrollState()
	var expanded by rememberSaveable { mutableStateOf(false) }
	val context = LocalContext.current
	val interactionSource = remember { MutableInteractionSource() }

	var showSaveChangesDialog by remember { mutableStateOf(false) }
	var customDnsDraft by rememberSaveable { mutableStateOf(customDns) }

	LaunchedEffect(customDns) {
		customDnsDraft = customDns
	}

	var lastSavedDns by rememberSaveable { mutableStateOf(customDns) }
	LaunchedEffect(customDns) {
		lastSavedDns = customDns
	}

	var lastSavedEnabled by rememberSaveable { mutableStateOf(dnsEnabled) }
	LaunchedEffect(dnsEnabled) {
		lastSavedEnabled = dnsEnabled
	}

	val hasUnsavedListChanges by remember(customDnsDraft, lastSavedDns) {
		derivedStateOf { customDnsDraft != lastSavedDns }
	}

	val hasUnsavedToggleChange by remember(dnsEnabled, lastSavedEnabled) {
		derivedStateOf { dnsEnabled != lastSavedEnabled }
	}

	fun leaveScreenWithReconnectIfNeeded(currentList: List<String>) {
		if (shouldReconnect(initialDnsEnabled, initialCustomDns, dnsEnabled, currentList)) {
			onReconnect()
		}
		onNavigateBack()
	}

	fun requestBack() {
		if (hasUnsavedListChanges || hasUnsavedToggleChange) {
			showSaveChangesDialog = true
		} else {
			leaveScreenWithReconnectIfNeeded(lastSavedDns)
		}
	}

	BackHandler { requestBack() }

	LaunchedEffect(onBackClickEventTriggered) {
		if (onBackClickEventTriggered) requestBack()
	}

	Column(
		horizontalAlignment = Alignment.Start,
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(scrollState)
			.padding(horizontal = 24.dp.scaledWidth())
			.navigationBarsPadding()
			.imePadding(),
	) {
		Text(
			text = stringResource(R.string.dns_description),
			style = Typography.bodyMedium,
			color = MaterialTheme.colorScheme.outline,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 16.dp),
		)

		Text(
			text = stringResource(R.string.censorship_quic_changes),
			style = MaterialTheme.typography.bodySmall,
			color = CustomColors.warning,
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 16.dp),
		)

		Text(
			text = stringResource(if (expanded) R.string.dns_hide_list else R.string.dns_view_list),
			style = Typography.bodyMedium.copy(textDecoration = TextDecoration.Underline),
			color = MaterialTheme.colorScheme.outline,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 24.dp)
				.clickable(
					role = Role.Button,
					indication = null,
					interactionSource = remember { MutableInteractionSource() },
				) { expanded = !expanded },
		)

		AnimatedVisibility(
			visible = expanded,
			enter = expandVertically(expandFrom = Alignment.Top) + fadeIn(),
			exit = shrinkVertically(shrinkTowards = Alignment.Top) + fadeOut(),
		) {
			Column(
				modifier = Modifier
					.fillMaxWidth()
					.padding(top = 24.dp),
			) {
				Text(
					text = stringResource(R.string.dns_list_title),
					style = Typography.bodyMedium,
					color = MaterialTheme.colorScheme.outline,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)

				Spacer(Modifier.height(4.dp))

				defaultDns.forEach { dns ->
					Row(
						modifier = Modifier
							.fillMaxWidth()
							.padding(vertical = 4.dp),
						verticalAlignment = Alignment.Top,
					) {
						Text(
							text = "•",
							style = Typography.bodyMedium,
							color = MaterialTheme.colorScheme.outline,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
							modifier = Modifier.padding(end = 8.dp),
						)
						Text(
							text = dns,
							style = Typography.bodyMedium,
							color = MaterialTheme.colorScheme.outline,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						)
					}
				}
			}
		}

		Spacer(Modifier.height(24.dp))

		CustomDnsCard(
			initialDns = lastSavedDns,
			dnsEnabled = dnsEnabled,
			onDnsEnable = onDnsEnable,
			onSave = { listToSave ->
				onSave(listToSave)
				lastSavedDns = listToSave
				customDnsDraft = listToSave
			},
			onDnsListChange = { customDnsDraft = it },
		)

		Spacer(Modifier.height(24.dp))

		Row(
			verticalAlignment = Alignment.CenterVertically,
			modifier = Modifier.clickable(interactionSource = interactionSource, indication = null) {
				context.openWebUrl(context.getString(R.string.dns_learn_more_link))
			},
		) {
			Text(
				text = stringResource(R.string.dns_learn_more_text),
				style = MaterialTheme.typography.bodyMedium.copy(textDecoration = TextDecoration.Underline),
				color = MaterialTheme.colorScheme.onBackground,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
			Spacer(modifier = Modifier.width(4.dp))
			Icon(
				imageVector = Icons.AutoMirrored.Outlined.OpenInNew,
				contentDescription = null,
				tint = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.size(12.dp),
			)
		}
	}

	SaveChangesModal(
		showSaveChangesDialog = showSaveChangesDialog,
		onClickSave = {
			val toSave = customDnsDraft
			onSave(toSave)
			lastSavedDns = toSave
			customDnsDraft = toSave
			lastSavedEnabled = dnsEnabled

			showSaveChangesDialog = false
			leaveScreenWithReconnectIfNeeded(toSave)
		},
		onDiscard = {
			showSaveChangesDialog = false
			onNavigateBack()
		},
		onDismiss = { showSaveChangesDialog = false },
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewDnsScreen() {
	NymVPNTheme(Theme.default()) {
		val list = arrayListOf<String>().apply {
			repeat(6) { add("192.0.2.44") }
			add("2606:4700:4700::1111")
		}

		DnsScreen(
			defaultDns = list,
			customDns = arrayListOf(),
			dnsEnabled = true,
			onDnsEnable = {},
			onSave = {},
			onBackClickEventTriggered = false,
			onNavigateBack = {},
			onReconnect = {},
			initialDnsEnabled = true,
			initialCustomDns = emptyList(),
		)
	}
}
