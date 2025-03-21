package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.ui.screens.settings.legal.licenses.components.LicensesList
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun LicensesScreen(appViewModel: AppViewModel, viewModel: LicensesViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val navController = LocalNavController.current
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
	val sortedLicenses = remember(licenses, licenseComparator) {
		licenses.sortedWith(licenseComparator)
	}

	LaunchedEffect(Unit) {
		viewModel.loadLicensesFromAssets()
	}

	LicensesList(
		licenses = sortedLicenses,
		modifier = Modifier
			.fillMaxSize()
			.padding(horizontal = 24.dp.scaledWidth())
			.windowInsetsPadding(WindowInsets.navigationBars),
	)
}
