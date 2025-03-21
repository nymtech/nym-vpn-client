package net.nymtech.nymvpn.ui.screens.settings.legal.licenses.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.screens.settings.legal.licenses.Artifact
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun LicensesList(licenses: List<Artifact>, modifier: Modifier = Modifier) {
	val context = LocalContext.current
	val snackbar = SnackbarController.current
	val listState = rememberLazyListState()

	LazyColumn(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Top),
		state = listState,
		modifier = modifier,
	) {
		item {
			Row(modifier = Modifier.padding(bottom = 24.dp.scaledHeight())) {}
		}
		itemsIndexed(
			items = licenses,
			key = { index, _ -> index },
			contentType = { _, _ -> null },
		) { _, artifact ->
			LicenseItem(
				artifact = artifact,
				onClick = {
					if (artifact.scm != null) {
						context.openWebUrl(artifact.scm.url)
					} else {
						snackbar.showMessage(context.getString(R.string.no_scm_found))
					}
				},
			)
		}
		item {
			Row(modifier = Modifier.padding(bottom = 24.dp.scaledHeight())) {}
		}
	}
}
