package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun LicensesScreen(appViewModel: AppViewModel, viewModel: LicensesViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val licenses by viewModel.licenses.collectAsStateWithLifecycle()

	LaunchedEffect(Unit) {
		viewModel.loadLicensesFromAssets()
	}

	LazyColumn(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Top),
		modifier =
		Modifier
			.fillMaxSize()
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		item {
			Row(modifier = Modifier.padding(bottom = 24.dp.scaledHeight())) {}
		}
		items(licenses) { it ->
			SurfaceSelectionGroupButton(
				items =
				listOf(
					SelectionItem(
						// TODO refactor
						title = {
							Text(
								if (it.name != null && it.name.length > 32) {
									it.name.substring(
										0,
										29,
									).plus("...")
								} else {
									it.name
										?: stringResource(id = R.string.unknown)
								},
								style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
							)
						},
						description = {
							Text(
								it.spdxLicenses?.joinToString(postfix = " ") { it.name } +
									if (it.unknownLicenses != null) {
										it.unknownLicenses.joinToString { it.name }
									} else {
										""
									},
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
							)
						},
						onClick = {
							if (it.scm != null) {
								appViewModel.openWebPage(it.scm.url)
							} else {
								appViewModel.showSnackbarMessage(
									context.getString(R.string.no_scm_found),
								)
							}
						},
					),
				),
			)
		}
		item {
			Row(modifier = Modifier.padding(bottom = 24.dp.scaledHeight())) {}
		}
	}
}
