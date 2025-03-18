package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun LicensesScreen(appViewModel: AppViewModel, viewModel: LicensesViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val navController = LocalNavController.current
	val snackbar = SnackbarController.current
	val licenses = viewModel.licenses

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle(stringResource(R.string.licenses)) },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			),
		)
	}

	val licenseComparator = compareBy<Artifact> { it.name?.lowercase() }

	val sortedLicenses =
		remember(licenses, licenseComparator) {
			licenses.sortedWith(licenseComparator)
		}

	LaunchedEffect(Unit) {
		viewModel.loadLicensesFromAssets()
	}

	fun buildLicenseText(artifact: Artifact): String {
		val spdxName = artifact.spdxLicenses?.map { it.name }
		val unknownNames = artifact.unknownLicenses?.map { it.name }
		val allNames = spdxName.orEmpty() + unknownNames.orEmpty()
		return allNames.distinct().joinToString()
	}

	LazyColumn(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Top),
		modifier =
		Modifier
			.fillMaxSize()
			.padding(horizontal = 24.dp.scaledWidth()).windowInsetsPadding(WindowInsets.navigationBars),
	) {
		item {
			Row(modifier = Modifier.padding(bottom = 24.dp.scaledHeight())) {}
		}
		itemsIndexed(sortedLicenses, key = { index, _ -> index }, contentType = { _: Int, _: Artifact -> null }) { _, artifact ->
			SurfaceSelectionGroupButton(
				items =
				listOf(
					SelectionItem(
						trailing = {
							val icon = Icons.AutoMirrored.Outlined.ArrowRight
							Icon(icon, stringResource(R.string.go), Modifier.size(iconSize))
						},
						title = {
							Text(
								artifact.name
									?: stringResource(id = R.string.unknown),
								style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
								maxLines = 1,
								overflow = TextOverflow.Ellipsis,
							)
						},
						description = {
							Text(
								text = buildLicenseText(artifact),
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
							)
						},
						onClick = {
							if (artifact.scm != null) {
								context.openWebUrl(artifact.scm.url)
							} else {
								snackbar.showMessage(
									context.getString(R.string.no_scm_found),
								)
							}
						},
					),
				),
				background = MaterialTheme.colorScheme.surface,
			)
		}
		item {
			Row(modifier = Modifier.padding(bottom = 24.dp.scaledHeight())) {}
		}
	}
}
