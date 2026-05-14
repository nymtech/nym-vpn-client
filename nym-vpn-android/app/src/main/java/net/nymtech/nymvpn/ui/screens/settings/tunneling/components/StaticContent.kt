package net.nymtech.nymvpn.ui.screens.settings.tunneling.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.screens.settings.tunneling.AppFilter
import net.nymtech.nymvpn.ui.screens.settings.tunneling.SplitTunnelingUiState
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun StaticContent(uiState: SplitTunnelingUiState, onQueryChange: (String) -> Unit, onSelectAllDirectAppsClick: () -> Unit, onSelectAllVpnPassThroughClick: () -> Unit) {
	Text(
		text = stringResource(R.string.split_tunneling_info_msg),
		style = MaterialTheme.typography.bodyMedium,
		color = MaterialTheme.colorScheme.onPrimaryContainer,
		modifier = Modifier.padding(top = 8.dp.scaledHeight()),
	)

	CustomTextField(
		value = uiState.query,
		onValueChange = onQueryChange,
		modifier = Modifier
			.fillMaxWidth()
			.padding(vertical = 24.dp.scaledHeight())
			.height(56.dp.scaledHeight())
			.padding(horizontal = 0.dp)
			.then(Modifier),
		placeholder = { Text(stringResource(R.string.split_tunneling_search_apps_hint), color = MaterialTheme.colorScheme.onBackground) },
		singleLine = true,
		leading = { Icon(Icons.Rounded.Search, contentDescription = stringResource(R.string.search), modifier = Modifier.height(iconSize)) },
		label = { Text(stringResource(R.string.search)) },
		textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onPrimaryContainer),
		showClearIcon = true,
	)

	Text(
		text = stringResource(R.string.split_tunneling_apps),
		style = MaterialTheme.typography.bodyMedium,
		color = MaterialTheme.colorScheme.onPrimaryContainer,
		fontWeight = FontWeight(500),
	)

	Row(
		modifier = Modifier
			.padding(top = 12.dp.scaledHeight(), bottom = 24.dp.scaledHeight())
			.fillMaxWidth(),
		horizontalArrangement = Arrangement.spacedBy(12.dp.scaledWidth()),
		verticalAlignment = Alignment.CenterVertically,
	) {
		FilterButton(
			title = stringResource(R.string.split_tunneling_direct),
			noOfApps = uiState.directAppsCount,
			description = stringResource(R.string.split_tunneling_direct_desc),
			imageVector = ImageVector.vectorResource(R.drawable.split),
			isSelected = uiState.appliedFilter == AppFilter.Direct,
			modifier = Modifier
				.weight(1f)
				.height(56.dp)
				.background(Color.Transparent, RoundedCornerShape(8.dp))
				.clickable { onSelectAllDirectAppsClick() },
		)
		FilterButton(
			title = stringResource(R.string.split_tunneling_via_vpn),
			noOfApps = uiState.vpnPassThroughAppsCount,
			description = stringResource(R.string.split_tunneling_via_desc),
			imageVector = ImageVector.vectorResource(R.drawable.shield),
			isSelected = uiState.appliedFilter == AppFilter.VpnPassThrough,
			modifier = Modifier
				.weight(1f)
				.height(56.dp)
				.background(Color.Transparent, RoundedCornerShape(8.dp))
				.clickable { onSelectAllVpnPassThroughClick() },
		)
	}

	Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
	HorizontalDivider(modifier = Modifier.fillMaxWidth(), color = MaterialTheme.colorScheme.surface.copy(alpha = 0.1f))
}
