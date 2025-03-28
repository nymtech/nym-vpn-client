package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.ui.screens.settings.legal.licenses.components.LicensesList
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun LicensesScreen(viewModel: LicensesViewModel = hiltViewModel()) {
	val licenses = viewModel.licenses

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
