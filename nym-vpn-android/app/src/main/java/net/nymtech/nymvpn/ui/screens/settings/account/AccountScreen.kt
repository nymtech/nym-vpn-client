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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.common.labels.GroupLabel
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.ui.screens.settings.account.model.Device
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.nymvpn.util.extensions.showToast

@Composable
fun AccountScreen(appViewModel: AppViewModel, appUiState: AppUiState) {
	val context = LocalContext.current
	val navController = LocalNavController.current

	val devicesDisabled = true

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle(stringResource(R.string.credential)) },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack) {
						navController.popBackStack()
					}
				},
			),
		)
	}

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
// 				appUiState.settings.credentialExpiry?.let {
// 					val credentialDuration = it.durationFromNow()
// 					val days = credentialDuration.toDaysPart()
// 					val hours = credentialDuration.toHoursPart()
// 					val durationLeft =
// 						buildAnnotatedString {
// 							append(days.toString())
// 							append(" ")
// 							append(if (days != 1L) stringResource(id = R.string.days) else stringResource(id = R.string.day))
// 							append(", ")
// 							append(hours.toString())
// 							append(" ")
// 							append(if (hours != 1) stringResource(id = R.string.hours) else stringResource(id = R.string.hour))
// 							append(" ")
// 							append(stringResource(id = R.string.remaining))
// 						}
// 					Text(
// 						durationLeft.text,
// 						style = CustomTypography.labelHuge,
// 						color = MaterialTheme.colorScheme.onSurface,
// 					)
// 					LinearProgressIndicator(
// 						modifier =
// 						Modifier
// 							.fillMaxWidth(),
// 						progress = {
// 							days.toFloat() / Constants.FREE_PASS_CRED_DURATION
// 						},
// 					)
// 				}

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
								navController.navigate(Route.Credential)
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
						context.showToast(R.string.feature_in_progress)
					}, modifier = Modifier.padding(start = 24.dp)) {
						Icon(
							Icons.Filled.Add,
							Icons.Filled.Add.name,
							tint = MaterialTheme.colorScheme.onSurface,
						)
					}
				}
				SurfaceSelectionGroupButton(
					// TODO add device to state when we add this feature
					items =
					emptyList<Device>().map {
						SelectionItem(
							{
								val icon = ImageVector.vectorResource(it.type.icon())
								Icon(
									icon,
									icon.name,
									modifier = Modifier.size(iconSize.scaledWidth()),
								)
							},
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
					background = MaterialTheme.colorScheme.surface,
				)
			}
		}
	}
}
