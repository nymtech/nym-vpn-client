package net.nymtech.nymvpn.ui.screens.account.delete

import android.content.res.Configuration
import androidx.annotation.StringRes
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.TransparentButton
import net.nymtech.nymvpn.ui.common.deviceauth.DeviceAuthAction
import net.nymtech.nymvpn.ui.common.deviceauth.rememberDeviceAuth
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.account.delete.components.DeleteSection
import net.nymtech.nymvpn.ui.screens.account.delete.components.PlayStopContent
import net.nymtech.nymvpn.ui.screens.account.delete.modal.DeleteConfirmDialog
import net.nymtech.nymvpn.ui.screens.account.delete.modal.DeleteErrorDialog
import net.nymtech.nymvpn.ui.screens.account.delete.modal.DeletingDialog
import net.nymtech.nymvpn.ui.screens.settings.geoexclusion.components.WarningCard
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.safePopBackStack
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun DeleteScreen(viewModel: DeleteViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val context = LocalContext.current
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()

	LifecycleEventEffect(Lifecycle.Event.ON_RESUME) { viewModel.checkSubscription() }
	LifecycleEventEffect(Lifecycle.Event.ON_STOP) { viewModel.dismissConfirm() }

	LaunchedEffect(uiState.step) {
		if (uiState.step == DeleteStep.DELETED) {
			navController.navigate(Route.DeletedAccount) {
				popUpTo(0) { inclusive = true }
				launchSingleTop = true
			}
		}
	}

	val deviceAuth = rememberDeviceAuth { _, _ -> viewModel.showConfirm() }

	DeleteConfirmDialog(
		show = uiState.showConfirm,
		onConfirm = viewModel::deleteAccount,
		onKeep = {
			viewModel.dismissConfirm()
			navController.safePopBackStack()
		},
	)
	DeletingDialog(show = uiState.isDeleting)
	DeleteErrorDialog(show = uiState.showError, onBack = viewModel::dismissError)

	if (uiState.isLoading) return

	val onOpenPlayClick: () -> Unit = { context.openWebUrl(Constants.URL_PLAY_SUBSCRIPTIONS + context.packageName) }

	when (uiState.step) {
		DeleteStep.PLAY_STOP -> PlayStopContent(
			onOpenPlayClick = onOpenPlayClick,
			onKeepClick = { navController.safePopBackStack() },
		)
		DeleteStep.EXPLAINER, DeleteStep.DELETED -> DeleteScreen(
			uiState = uiState,
			onOpenPlayClick = onOpenPlayClick,
			onDeleteClick = { deviceAuth.request(DeviceAuthAction.DELETE_ACCOUNT) },
			onKeepClick = { navController.safePopBackStack() },
		)
	}
}

@Composable
fun DeleteScreen(uiState: DeleteUiState, onOpenPlayClick: () -> Unit, onDeleteClick: () -> Unit, onKeepClick: () -> Unit) {
	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(rememberScrollState())
			.padding(horizontal = 24.dp.scaledWidth(), vertical = 24.dp.scaledHeight())
			.navigationBarsPadding(),
	) {
		if (uiState.isPlayStatusUnknown) {
			Column {
				WarningCard(
					title = stringResource(R.string.delete_account_play_title),
					text = stringResource(R.string.delete_account_play_descr_1),
				)
				TransparentButton(
					onClick = onOpenPlayClick,
					content = {
						Text(
							stringResource(R.string.delete_account_play_button),
							style = MaterialTheme.typography.bodyLarge,
							color = MaterialTheme.colorScheme.onPrimaryContainer,
						)
					},
					modifier = Modifier
						.fillMaxWidth()
						.height(48.dp.scaledHeight()),
				)
			}
		}
		uiState.timeLeftUntil?.let { date ->
			WarningCard(
				title = stringResource(R.string.delete_account_time_title),
				text = stringResource(R.string.delete_account_time_descr, date),
			)
		}
		Text(
			text = stringResource(R.string.delete_account_headline),
			style = MaterialTheme.typography.bodyLarge,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
		)
		DeleteSection(
			title = stringResource(R.string.delete_account_deleted_title),
			items = listOf(
				AnnotatedString(stringResource(R.string.delete_account_deleted_1)),
				AnnotatedString(stringResource(R.string.delete_account_deleted_2)),
				AnnotatedString(stringResource(R.string.delete_account_deleted_3)),
				AnnotatedString(stringResource(R.string.delete_account_deleted_4)),
			),
		)
		DeleteSection(
			title = stringResource(R.string.delete_account_at_nym_title),
			items = listOf(AnnotatedString(stringResource(R.string.delete_account_at_nym_1))),
		)
		DeleteSection(
			title = stringResource(R.string.delete_account_kept_title),
			items = listOfNotNull(
				keptItem(R.string.delete_account_kept_1_headline, R.string.delete_account_kept_1_descr),
				if (uiState.showPlayRecords) keptItem(R.string.delete_account_kept_2_headline, R.string.delete_account_kept_2_descr) else null,
				keptItem(R.string.delete_account_kept_3_headline, R.string.delete_account_kept_3_descr),
				keptItem(R.string.delete_account_kept_4_headline, R.string.delete_account_kept_4_descr),
				keptItem(R.string.delete_account_kept_5_headline, R.string.delete_account_kept_5_descr),
				keptItem(R.string.delete_account_kept_6_headline, R.string.delete_account_kept_6_descr),
			),
		)
		Column {
			MainStyledButton(
				onClick = onDeleteClick,
				textColor = MaterialTheme.colorScheme.onPrimaryContainer,
				content = {
					Text(
						stringResource(R.string.account_info_delete_button),
						style = MaterialTheme.typography.bodyLarge,
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(48.dp.scaledHeight()),
				color = LocalNymColors.current.buttonErrorText,
				borderStroke = BorderStroke(width = 1.dp, color = LocalNymColors.current.buttonErrorBorder),
			)
			Spacer(Modifier.height(8.dp))
			TransparentButton(
				onClick = onKeepClick,
				content = {
					Text(
						stringResource(R.string.delete_account_keep_button),
						style = MaterialTheme.typography.bodyLarge,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(48.dp.scaledHeight()),
			)
		}
	}
}

@Composable
private fun keptItem(@StringRes headline: Int, @StringRes description: Int): AnnotatedString {
	val headlineText = stringResource(headline)
	val descriptionText = stringResource(description)
	return buildAnnotatedString {
		withStyle(SpanStyle(fontWeight = FontWeight.Bold)) {
			append(headlineText)
		}
		append(" ")
		append(descriptionText)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewDeleteScreen() {
	NymVPNTheme(Theme.default()) {
		DeleteScreen(
			uiState = DeleteUiState(isLoading = false, showPlayRecords = true, isPlayStatusUnknown = true, timeLeftUntil = "14 February 2027"),
			onOpenPlayClick = {},
			onDeleteClick = {},
			onKeepClick = {},
		)
	}
}
