package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SecondaryTabRow
import androidx.compose.material3.Tab
import androidx.compose.material3.TabRowDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R

@Composable
fun LogsPagerTabs(selectedTab: Int, onSelectTab: (Int) -> Unit, modifier: Modifier = Modifier) {
	val tabs = listOf(
		stringResource(R.string.logs_tab_app),
		stringResource(R.string.logs_tab_tunnel),
		stringResource(R.string.logs_tab_library),
	)

	SecondaryTabRow(
		selectedTabIndex = selectedTab,
		modifier = modifier.fillMaxWidth(),
		containerColor = MaterialTheme.colorScheme.primaryContainer,
		contentColor = MaterialTheme.colorScheme.onPrimaryContainer,
		divider = {},
		indicator = {
			TabRowDefaults.SecondaryIndicator(
				modifier = Modifier
					.tabIndicatorOffset(selectedTabIndex = selectedTab, matchContentSize = false)
					.height(2.dp),
				color = MaterialTheme.colorScheme.primary,
			)
		},
	) {
		tabs.forEachIndexed { index, label ->
			val selected = index == selectedTab
			Tab(
				selected = selected,
				onClick = { onSelectTab(index) },
				text = {
					Text(
						text = label,
						style = MaterialTheme.typography.labelLarge,
						color = if (selected) {
							MaterialTheme.colorScheme.onPrimaryContainer
						} else {
							MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.6f)
						},
					)
				},
			)
		}
	}
}
