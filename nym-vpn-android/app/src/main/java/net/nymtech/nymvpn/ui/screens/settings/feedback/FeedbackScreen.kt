package net.nymtech.nymvpn.ui.screens.settings.feedback

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun FeedbackScreen(appViewModel: AppViewModel, viewModel: FeedbackViewModel = hiltViewModel()) {

    val isErrorReportingEnabled by viewModel.isErrorReportingEnabled.collectAsStateWithLifecycle()
    var showErrorReportingDialog by remember { mutableStateOf(false) }


    val context = LocalContext.current

    AnimatedVisibility(showErrorReportingDialog) {
        AlertDialog(
            containerColor = CustomColors.snackBarBackgroundColor,
            onDismissRequest = { showErrorReportingDialog = false },
            confirmButton = {
                TextButton(
                    onClick = {
                        showErrorReportingDialog = false
                        viewModel.onErrorReportingSelected(!isErrorReportingEnabled)
                    },
                ) {
                    Text(text = stringResource(R.string.okay))
                }
            },
            dismissButton = {
                TextButton(onClick = { showErrorReportingDialog = false }) {
                    Text(text = stringResource(R.string.cancel))
                }
            },
            title = {
                Text(
                    text = stringResource(R.string.error_reporting),
                    color = CustomColors.snackbarTextColor
                )
            },
            text = {
                Text(
                    text = stringResource(R.string.error_reporting_alert),
                    color = CustomColors.snackbarTextColor
                )
            },
        )
    }

    Column(
        horizontalAlignment = Alignment.Start,
        verticalArrangement = Arrangement.spacedBy(24.dp, Alignment.Top),
        modifier =
        Modifier
            .verticalScroll(rememberScrollState())
            .fillMaxSize()
            .padding(top = 24.dp.scaledHeight())
            .padding(horizontal = 24.dp.scaledWidth())
    ) {
        SurfaceSelectionGroupButton(
            listOf(
                SelectionItem(
                    leadingIcon = ImageVector.vectorResource(R.drawable.github),
                    title = stringResource(R.string.open_github),
                    onClick = { appViewModel.openWebPage(context.getString(R.string.github_issues_url)) }),
            )
        )
        SurfaceSelectionGroupButton(
            listOf(
                SelectionItem(
                    leadingIcon = ImageVector.vectorResource(R.drawable.send),
                    title = stringResource(R.string.send_feedback),
                    onClick = {
                        appViewModel.launchEmail()
                    })
            )
        )
        SurfaceSelectionGroupButton(
            listOf(
                SelectionItem(
                    leadingIcon = ImageVector.vectorResource(R.drawable.matrix),
                    title = stringResource(R.string.join_matrix),
                    onClick = {
                        appViewModel.openWebPage(context.getString(R.string.matrix_url))
                    }),
            )
        )
        SurfaceSelectionGroupButton(
            listOf(
                SelectionItem(
                    leadingIcon = ImageVector.vectorResource(R.drawable.discord),
                    title = stringResource(R.string.join_discord),
                    onClick = {
                        appViewModel.openWebPage(context.getString(R.string.discord_url))
                    }),
            )
        )
        SurfaceSelectionGroupButton(
            listOf(
                SelectionItem(
                    ImageVector.vectorResource(R.drawable.error),
                    title = stringResource(R.string.error_reporting),
                    description = stringResource(R.string.error_reporting_description),
                    trailing = {
                        Switch(
                            isErrorReportingEnabled,
                            { showErrorReportingDialog = true },
                            modifier = Modifier
                                .height(32.dp.scaledHeight())
                                .width(52.dp.scaledWidth())
                        )
                    })
            )
        )
    }
}
