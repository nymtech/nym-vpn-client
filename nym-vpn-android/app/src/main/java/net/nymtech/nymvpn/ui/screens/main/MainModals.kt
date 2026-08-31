package net.nymtech.nymvpn.ui.screens.main

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.snackbar.AlertController
import net.nymtech.nymvpn.ui.common.snackbar.AlertMessage
import net.nymtech.nymvpn.ui.common.snackbar.AlertType
import net.nymtech.nymvpn.ui.screens.account.info.AutologinState
import net.nymtech.nymvpn.ui.screens.account.info.modal.AutologinLoadingDialog
import net.nymtech.nymvpn.ui.screens.account.info.modal.PinCodeDialog
import net.nymtech.nymvpn.ui.screens.main.bottomsheet.MainBottomSheet
import net.nymtech.nymvpn.ui.screens.main.bottomsheet.MainBottomSheetContent
import net.nymtech.nymvpn.ui.screens.main.modal.BatteryModal
import net.nymtech.nymvpn.ui.screens.main.modal.CompatibilityModal
import net.nymtech.nymvpn.ui.screens.main.modal.NetworkStatsModal
import net.nymtech.nymvpn.ui.screens.main.modal.NodeFamiliesModal
import net.nymtech.nymvpn.ui.screens.main.modal.ShowInfoModal

@Composable
fun MainModals(
	autologinState: AutologinState,
	showInfoDialog: Boolean,
	showCompatibilityDialog: Boolean,
	showBatteryDialog: Boolean,
	showNetworkStatsDialog: Boolean,
	showNodeFamiliesDialog: Boolean,
	bottomSheetContent: MainBottomSheetContent,
	onCancelAutologin: () -> Unit,
	onDismissAutologin: () -> Unit,
	onDismissInfo: () -> Unit,
	onDismissCompatibility: () -> Unit,
	onConfirmCompatibility: () -> Unit,
	onClickBatterySettings: () -> Unit,
	onDismissBattery: () -> Unit,
	onConfirmStats: () -> Unit,
	onDismissStats: () -> Unit,
	onConfirmNodeFamilies: () -> Unit,
	onDismissNodeFamilies: () -> Unit,
	onNotificationSettingsClick: () -> Unit,
	onDismissBottomSheet: () -> Unit,
	onAuthSuccess: () -> Unit,
	onLoginProcessingStart: (passphrase: String) -> Unit,
	authSheetMinHeightPx: Int = 0,
	onAuthSheetHeightChange: (Int) -> Unit = {},
	appUiState: AppUiState,
) {
	val context = LocalContext.current

	when (val autologin = autologinState) {
		is AutologinState.Loading -> AutologinLoadingDialog(onCancel = onCancelAutologin)
		is AutologinState.PinReady -> PinCodeDialog(
			pinCode = autologin.pinCode,
			url = autologin.url,
			onDismiss = onDismissAutologin,
		)
		else -> Unit
	}

	ShowInfoModal(
		context = context,
		showInfoDialog = showInfoDialog,
		onDismiss = onDismissInfo,
	)

	CompatibilityModal(
		showCompatibilityDialog = showCompatibilityDialog,
		onDismiss = onDismissCompatibility,
		onConfirmClick = onConfirmCompatibility,
	)

	val batteryOptTitle = stringResource(R.string.battery_opt_settings_text)
	BatteryModal(
		showBatteryDialog = showBatteryDialog,
		onClickSettings = onClickBatterySettings,
		onDismiss = {
			AlertController.show(AlertMessage(type = AlertType.Neutral, title = batteryOptTitle))
			onDismissBattery()
		},
	)

	val statsEnabledTitle = stringResource(R.string.notification_stats_enabled)
	NetworkStatsModal(
		showNetworkStatsDialog = showNetworkStatsDialog,
		onConfirm = {
			AlertController.show(AlertMessage(type = AlertType.Confirmation, title = statsEnabledTitle))
			onConfirmStats()
		},
		onDismiss = onDismissStats,
	)

	NodeFamiliesModal(
		showDialog = showNodeFamiliesDialog,
		onConfirmClick = onConfirmNodeFamilies,
		onNotificationSettingsClick = onNotificationSettingsClick,
		onDismiss = onDismissNodeFamilies,
	)

	MainBottomSheet(
		content = bottomSheetContent,
		onDismissRequest = onDismissBottomSheet,
		onAuthSuccess = onAuthSuccess,
		onLoginProcessingStart = onLoginProcessingStart,
		authSheetMinHeightPx = authSheetMinHeightPx,
		onAuthSheetHeightChange = onAuthSheetHeightChange,
		appUiState = appUiState,
	)
}
