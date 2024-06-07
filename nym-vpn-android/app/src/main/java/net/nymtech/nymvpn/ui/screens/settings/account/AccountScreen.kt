package net.nymtech.nymvpn.ui.screens.settings.account

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.common.labels.GroupLabel
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.durationFromNow
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun AccountScreen(appViewModel: AppViewModel, appUiState: AppUiState, navController: NavController, viewModel: AccountViewModel = hiltViewModel()) {
	val context = LocalContext.current

	val uiState by viewModel.uiState.collectAsStateWithLifecycle()

	val devicesDisabled = true

	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier =
		Modifier
			.fillMaxSize()
			.padding(horizontal = 24.dp.scaledHeight()),
	) {
		Box(
			modifier =
			Modifier
				.height(IntrinsicSize.Min)
				.fillMaxWidth()
				.padding(vertical = 16.dp.scaledHeight()),
		) {
			Column(
				verticalArrangement = Arrangement.spacedBy(16.dp.scaledHeight()),
				horizontalAlignment = Alignment.Start,
				modifier = Modifier.fillMaxSize(),
			) {
				appUiState.credentialExpiryTime?.let {
					val credentialDuration = it.durationFromNow()
					val days = credentialDuration.toDaysPart()
					val hours = credentialDuration.toHoursPart()
					val durationLeft =
						buildAnnotatedString {
							append(days.toString())
							append(" ")
							append(if (days != 1L) stringResource(id = R.string.days) else stringResource(id = R.string.day))
							append(", ")
							append(hours.toString())
							append(" ")
							append(if (hours != 1) stringResource(id = R.string.hours) else stringResource(id = R.string.hour))
							append(" ")
							append(stringResource(id = R.string.left))
						}
					Text(
						durationLeft.text,
						style = CustomTypography.labelHuge,
						color = MaterialTheme.colorScheme.onSurface,
					)
					LinearProgressIndicator(
						modifier =
						Modifier
							.fillMaxWidth(),
						progress = {
							// TODO need to think about this more, setting to full for now
							1f
						},
					)
				}

				Row(
					horizontalArrangement = Arrangement.SpaceBetween,
					verticalAlignment = Alignment.CenterVertically,
					modifier =
					Modifier
						.heightIn(min = 40.dp.scaledHeight())
						.fillMaxWidth(),
				) {
					Text(
						stringResource(id = R.string.top_up_credential),
						style = MaterialTheme.typography.bodyLarge,
						color = MaterialTheme.colorScheme.onSurfaceVariant,
						modifier = Modifier.padding(end = 24.dp.scaledWidth()),
					)
					Box(modifier = Modifier.width(100.dp.scaledWidth())) {
						MainStyledButton(
							onClick = {
								navController.navigate(NavItem.Settings.Credential.route)
							},
							content = {
								Text(
									stringResource(id = R.string.top_up),
									style = CustomTypography.labelHuge,
								)
							},
						)
					}
				}
			}
		}
		if (!devicesDisabled) {
			Column(
				verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Top),
				modifier = Modifier.fillMaxSize(),
			) {
				Row(
					horizontalArrangement = Arrangement.SpaceBetween,
					verticalAlignment = Alignment.CenterVertically,
					modifier =
					Modifier
						.height(48.dp)
						.fillMaxWidth(),
				) {
					GroupLabel(title = stringResource(R.string.devices))
					IconButton(onClick = {
						appViewModel.showFeatureInProgressMessage(context)
					}, modifier = Modifier.padding(start = 24.dp)) {
						Icon(
							Icons.Filled.Add,
							Icons.Filled.Add.name,
							tint = MaterialTheme.colorScheme.onSurface,
						)
					}
				}
				SurfaceSelectionGroupButton(
					items =
					uiState.devices.map {
						SelectionItem(
							ImageVector.vectorResource(it.type.icon()),
							trailing = {
								IconButton(
									onClick = { /*TODO handle item delete from authorized*/ },
								) {
									Icon(Icons.Filled.Clear, Icons.Filled.Clear.name)
								}
							},
							title = { Text(it.name, style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
							description = {
								Text(it.type.formattedName().asString(context), style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline))
							},
						)
					},
				)
			}
		}
	}
}
