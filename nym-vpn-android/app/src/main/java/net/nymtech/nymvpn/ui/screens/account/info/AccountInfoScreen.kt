package net.nymtech.nymvpn.ui.screens.account.info

import android.content.ClipData
import android.content.res.Configuration
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Devices
import androidx.compose.material.icons.outlined.Numbers
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.toClipEntry
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.screens.account.info.components.AccountActionCard
import net.nymtech.nymvpn.ui.screens.account.info.components.AccountInfoCard
import net.nymtech.nymvpn.ui.screens.account.info.components.BandwidthUiState
import net.nymtech.nymvpn.ui.screens.account.info.modal.AutologinLoadingDialog
import net.nymtech.nymvpn.ui.screens.account.info.modal.PinCodeDialog
import net.nymtech.nymvpn.ui.screens.account.info.components.SubscriptionSection
import net.nymtech.nymvpn.ui.screens.settings.components.ExpiryState
import net.nymtech.nymvpn.ui.screens.settings.components.LogoutDialog
import net.nymtech.nymvpn.ui.screens.settings.components.LogoutSection
import net.nymtech.nymvpn.ui.screens.settings.components.SubscriptionStatusText
import net.nymtech.nymvpn.ui.screens.settings.components.SubscriptionUiState
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.StringValue
import nym_vpn_lib_types.DeeplinkKind
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledWidth
import timber.log.Timber

@Composable
fun AccountInfoScreen(appViewModel: AppViewModel, appUiState: AppUiState, viewModel: AccountInfoViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val context = LocalContext.current
	val scope = rememberCoroutineScope()
	val clipboardManager = LocalClipboard.current

	var loggingOut by remember { mutableStateOf(false) }
	var showLogoutDialog by remember { mutableStateOf(false) }
	val supportURL = stringResource(R.string.contact_url)

	val uiState by viewModel.uiState.collectAsStateWithLifecycle()

	LaunchedEffect(appUiState.managerState.isMnemonicStored) {
		loggingOut = false
		showLogoutDialog = false
	}

	LogoutDialog(
		show = showLogoutDialog,
		isLoggingOut = loggingOut,
		onDismiss = { showLogoutDialog = false },
		onConfirm = {
			loggingOut = true
			appViewModel.logout {
				navController.navigate(Route.Main()) {
					popUpTo(0) { inclusive = true }
					launchSingleTop = true
				}
			}
		},
	)

	when (val autologin = uiState.autologin) {
		is AutologinState.Loading -> AutologinLoadingDialog(onCancel = viewModel::cancelAutologin)
		is AutologinState.PinReady -> PinCodeDialog(
			pinCode = autologin.pinCode,
			url = autologin.url,
			onDismiss = viewModel::dismissAutologin,
		)
		is AutologinState.Error -> {
			SnackbarController.showMessage(StringValue.StringResource(R.string.account_info_autologin_error))
		}
		AutologinState.Idle -> {}
	}

	AccountInfoScreenContent(
		accountId = uiState.accountId,
		deviceId = uiState.deviceId,
		showLinkAccount = uiState.showLinkAccount,
		isMnemonicStored = uiState.isMnemonicStored,
		subscriptionState = uiState.subscription,
		bandwidthState = uiState.bandwidth,
		onManageClick = { viewModel.fetchAutologin(DeeplinkKind.AUTOLOGIN_VIEW) },
		onRenewClick = { viewModel.fetchAutologin(DeeplinkKind.AUTOLOGIN_RENEW) },
		onLinkAccountClick = {
			uiState.accountLinkUrl?.let {
				Timber.d("Link url: $it")
				context.openWebUrl(it)
			}
		},
		onAccountIdClick = {
			if (uiState.accountId.isNotEmpty()) {
				scope.launch {
					val clipData = ClipData.newPlainText("Account ID", uiState.accountId)
					clipboardManager.setClipEntry(clipData.toClipEntry())
				}
			}
		},
		onDeviceIdClick = {
			if (uiState.deviceId.isNotEmpty()) {
				scope.launch {
					val clipData = ClipData.newPlainText("Device ID", uiState.deviceId)
					clipboardManager.setClipEntry(clipData.toClipEntry())
				}
			}
		},
		onLogoutClick = {
			showLogoutDialog = true
		},
		onSelectPlanClick = {
			navController.goFromRoot(Route.SelectPlan)
		},
		onContactSupportClick = {
			context.openWebUrl(supportURL)
		},
	)
}

@Composable
fun AccountInfoScreenContent(
	accountId: String,
	deviceId: String,
	showLinkAccount: Boolean,
	isMnemonicStored: Boolean,
	onManageClick: () -> Unit,
	onRenewClick: () -> Unit,
	onLinkAccountClick: () -> Unit,
	onAccountIdClick: () -> Unit,
	onDeviceIdClick: () -> Unit,
	onLogoutClick: () -> Unit,
	onSelectPlanClick: () -> Unit,
	onContactSupportClick: () -> Unit,
	subscriptionState: SubscriptionUiState?,
	bandwidthState: BandwidthUiState?,
) {
	Column(
		horizontalAlignment = Alignment.Start,
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(rememberScrollState())
			.padding(horizontal = 16.dp.scaledWidth(), vertical = 24.dp),
	) {
		SubscriptionSection(
			subscriptionState = subscriptionState,
			bandwidthState = bandwidthState,
			onSelectPlanClick = onSelectPlanClick,
			onRenewClick = onRenewClick,
			onContactSupportClick = onContactSupportClick,
		)

		Spacer(Modifier.height(24.dp))

		AccountActionCard(
			title = stringResource(R.string.account_info_manage_button),
			subtitle = {
				SubscriptionStatusText(subscription = subscriptionState)
			},
			icon = Icons.Outlined.Person,
			onClick = onManageClick,
		)

		Spacer(Modifier.height(8.dp))
		AccountActionCard(
			title = stringResource(R.string.account_info_account_button),
			subtitle = if (showLinkAccount) {
				{
					Text(
						text = stringResource(R.string.account_info_link_description),
						style = MaterialTheme.typography.bodySmall.copy(MaterialTheme.colorScheme.outline),
					)
				}
			} else {
				null
			},
			icon = Icons.Outlined.Person,
			onClick = onLinkAccountClick,
		)

		Text(
			text = stringResource(if (showLinkAccount) R.string.account_info_link_warning else R.string.account_info_linked_info),
			style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
			modifier = Modifier.padding(vertical = 24.dp),
		)

		AccountInfoCard(
			title = stringResource(R.string.account_info_id_title),
			value = accountId,
			icon = Icons.Outlined.Numbers,
			onClick = onAccountIdClick,
		)

		Text(
			text = stringResource(R.string.account_info_id_info),
			style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
			modifier = Modifier.padding(vertical = 24.dp),
		)

		AccountInfoCard(
			title = stringResource(R.string.account_info_device_title),
			value = deviceId,
			icon = Icons.Outlined.Devices,
			onClick = onDeviceIdClick,
		)

		Text(
			text = stringResource(R.string.account_info_device_info),
			style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
			modifier = Modifier.padding(vertical = 24.dp),
		)

		LogoutSection(isMnemonicStored, onLogoutClick)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewAccountInfoScreen() {
	NymVPNTheme(Theme.default()) {
		AccountInfoScreenContent(
			accountId = "AccountID",
			deviceId = "DeviceID123",
			showLinkAccount = true,
			isMnemonicStored = true,
			onManageClick = {},
			onRenewClick = {},
			onLinkAccountClick = {},
			onAccountIdClick = {},
			onDeviceIdClick = {},
			onLogoutClick = {},
			onSelectPlanClick = {},
			onContactSupportClick = {},
			subscriptionState = SubscriptionUiState(
				isRecurring = false,
				validUntilDate = "December 24, 2026",
				expiryState = ExpiryState.NORMAL,
			),
			bandwidthState = BandwidthUiState(
				consumedGb = 800f,
				totalGb = 2000f,
				percentage = 0.4f,
				resetDate = "2026.03.18",
			),
		)
	}
}
