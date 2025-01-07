package net.nymtech.nymvpn.ui.screens.settings.support

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material3.Icon
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
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SupportScreen(appViewModel: AppViewModel) {
	val context = LocalContext.current
	val navController = LocalNavController.current

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle(stringResource(R.string.support)) },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack) {
						navController.popBackStack()
					}
				},
			),
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
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					leadingIcon = ImageVector.vectorResource(R.drawable.faq),
					{
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, icon.name, Modifier.size(iconSize))
					},
					title = { Text(stringResource(R.string.check_faq), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { context.openWebUrl(context.getString(R.string.faq_url)) },
				),
			),
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					leadingIcon = ImageVector.vectorResource(R.drawable.send),
					{
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, icon.name, Modifier.size(iconSize))
					},
					title = { Text(stringResource(R.string.get_in_touch), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = {
						context.openWebUrl(context.getString(R.string.contact_url))
					},
				),
			),
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					leadingIcon = ImageVector.vectorResource(R.drawable.github),
					{
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, icon.name, Modifier.size(iconSize))
					},
					title = { Text(stringResource(R.string.open_github), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = {
						context.openWebUrl(
							context.getString(R.string.github_issues_url),
						)
					},
				),
			),
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					leadingIcon = ImageVector.vectorResource(R.drawable.matrix),
					{
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, icon.name, Modifier.size(iconSize))
					},
					title = { Text(stringResource(R.string.join_matrix), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = {
						context.openWebUrl(context.getString(R.string.matrix_url))
					},
				),
			),
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					leadingIcon = ImageVector.vectorResource(R.drawable.discord),
					{
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, icon.name, Modifier.size(iconSize))
					},
					title = { Text(stringResource(R.string.join_discord), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = {
						context.openWebUrl(context.getString(R.string.discord_url))
					},
				),
			),
		)
	}
}
