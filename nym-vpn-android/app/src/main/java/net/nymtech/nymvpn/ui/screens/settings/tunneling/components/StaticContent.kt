package net.nymtech.nymvpn.ui.screens.settings.tunneling.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Shield
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
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.screens.settings.tunneling.AppFilter
import net.nymtech.nymvpn.ui.screens.settings.tunneling.SplitTunnelingUiState
import net.nymtech.nymvpn.ui.screens.settings.tunneling.UiEvent
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun StaticContent(uiState: SplitTunnelingUiState, onUiEvent: (UiEvent) -> Unit) {
	Text(
		text = stringResource(R.string.split_tunneling_info_msg),
		style = Typography.bodyMedium,
		color = MaterialTheme.colorScheme.outline,
		modifier = Modifier.padding(top = 8.dp.scaledHeight()),
		fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
	)
	Text(
		text = stringResource(R.string.split_tunneling_info_desc),
		modifier = Modifier.padding(top = 12.dp.scaledHeight()),
		style = Typography.bodyMedium,
		color = CustomColors.warning,
		fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
	)
	CustomTextField(
		value = uiState.query,
		onValueChange = { onUiEvent(UiEvent.QueryChange(it)) },
		modifier = Modifier
			.fillMaxWidth()
			.padding(vertical = 24.dp.scaledHeight())
			.height(56.dp.scaledHeight())
			.background(Color.Transparent, RoundedCornerShape(30.dp)),
		placeholder = { Text(stringResource(R.string.split_tunneling_search_apps_hint), color = MaterialTheme.colorScheme.outline) },
		singleLine = true,
		leading = { Icon(Icons.Rounded.Search, contentDescription = stringResource(R.string.search), modifier = Modifier.size(iconSize)) },
		label = { Text(stringResource(R.string.search)) },
		textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onSurface),
	)
	Text(
		text = stringResource(R.string.split_tunneling_apps),
		style = Typography.bodyLarge,
		color = MaterialTheme.colorScheme.onSurface,
		fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		fontWeight = FontWeight(500),
	)

	Row(
		modifier = Modifier
			.padding(top = 12.dp.scaledHeight(), bottom = 24.dp.scaledHeight())
			.fillMaxWidth()
			.height(IntrinsicSize.Min),
		horizontalArrangement = Arrangement.spacedBy(12.dp.scaledWidth()),
		verticalAlignment = Alignment.CenterVertically,
	) {
		FilterButton(
			stringResource(R.string.split_tunneling_direct),
			uiState.directAppsCount,
			stringResource(R.string.split_tunneling_direct_desc),
			ImageVector.vectorResource(R.drawable.split),
			isSelected = uiState.appliedFilter == AppFilter.Direct,
			modifier = Modifier
				.weight(1f)
				.fillMaxHeight()
				.clickable {
					onUiEvent(UiEvent.SelectAllDirectAppsClick)
				},
		)
		FilterButton(
			stringResource(R.string.split_tunneling_via_vpn),
			uiState.vpnPassThroughAppsCount,
			stringResource(R.string.split_tunneling_via_desc),
			Icons.Filled.Shield,
			isSelected = uiState.appliedFilter == AppFilter.VpnPassThrough,
			modifier = Modifier
				.weight(1f)
				.fillMaxHeight()
				.clickable {
					onUiEvent(UiEvent.SelectAllVpnPassThroughClick)
				},
		)
	}

	Row(
		modifier = Modifier
			.fillMaxWidth()
			.padding(bottom = 12.dp.scaledHeight()),
		horizontalArrangement = Arrangement.End,
	) {
		Text(
			text = stringResource(R.string.split_tunneling_direct),
			modifier = Modifier.widthIn(min = 50.dp),
			style = Typography.bodySmall.copy(fontSize = 12.sp, lineHeight = 12.sp),
			color = MaterialTheme.colorScheme.onSurface,
			textAlign = TextAlign.Center,
		)
		Spacer(modifier = Modifier.width(8.dp.scaledWidth()))
		Text(
			text = stringResource(R.string.split_tunneling_nym_vpn),
			modifier = Modifier.widthIn(min = 42.dp),
			style = Typography.bodySmall.copy(fontSize = 12.sp, lineHeight = 12.sp),
			color = MaterialTheme.colorScheme.onSurface,
			textAlign = TextAlign.Center,
		)
	}

	HorizontalDivider(modifier = Modifier.fillMaxWidth(), color = MaterialTheme.colorScheme.surface.copy(alpha = 0.1f))
}
