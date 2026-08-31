package net.nymtech.nymvpn.ui.screens.settings.appearance.appicon

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Palette
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.core.graphics.drawable.toBitmap
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.domain.AppIcon
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.IconSurfaceButton
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.TransparentButton
import net.nymtech.nymvpn.util.AppIconUtil
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel

@Composable
fun AppIconScreen(appUiState: AppUiState, viewModel: AppIconViewModel = hiltViewModel()) {
	val currentIcon by viewModel.currentIcon.collectAsState()
	val context = LocalContext.current

	AppIconScreen(
		currentIcon = currentIcon,
		canSwitch = appUiState.managerState.tunnelState == Tunnel.State.Down,
		onIconSelect = { AppIconUtil.apply(context, it) },
	)
}

@Composable
internal fun AppIconScreen(currentIcon: AppIcon, canSwitch: Boolean, onIconSelect: (AppIcon) -> Unit) {
	var pendingIcon by remember { mutableStateOf<AppIcon?>(null) }

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(16.dp.scaledHeight(), Alignment.Top),
		modifier = Modifier
			.fillMaxSize()
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 16.dp.scaledWidth()),
	) {
		if (!canSwitch) {
			Text(
				text = stringResource(R.string.app_icon_disconnect_first),
				color = MaterialTheme.colorScheme.error,
				style = MaterialTheme.typography.bodyMedium,
			)
		}

		LazyVerticalGrid(
			columns = GridCells.Fixed(2),
			horizontalArrangement = Arrangement.spacedBy(12.dp.scaledWidth()),
			verticalArrangement = Arrangement.spacedBy(12.dp.scaledHeight()),
			modifier = Modifier
				.fillMaxWidth()
				.alpha(if (canSwitch) 1f else 0.4f),
		) {
			items(AppIcon.entries) { icon ->
				val context = LocalContext.current
				val label = stringResource(icon.labelRes)
				val previewBitmap = remember(icon) {
					ContextCompat.getDrawable(context, icon.previewDrawable)
						?.toBitmap(width = 144, height = 144)
						?.asImageBitmap()
				}
				IconSurfaceButton(
					title = label,
					selected = currentIcon == icon,
					onClick = { if (canSwitch) pendingIcon = icon },
					leading = {
						previewBitmap?.let {
							Image(
								bitmap = it,
								contentDescription = label,
								modifier = Modifier.size(48.dp),
							)
						}
					},
				)
			}
		}
	}

	pendingIcon?.let { target ->
		Modal(
			show = true,
			onDismiss = { pendingIcon = null },
			icon = Icons.Outlined.Palette,
			title = {
				Text(
					text = stringResource(R.string.app_icon_change_title),
					style = MaterialTheme.typography.titleLarge,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
			},
			text = {
				Text(
					text = stringResource(R.string.app_icon_change_body),
					textAlign = TextAlign.Center,
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)
			},
			confirmButton = {
				MainStyledButton(
					onClick = {
						pendingIcon = null
						onIconSelect(target)
					},
					content = {
						Text(
							stringResource(R.string.app_icon_change_confirm),
							style = MaterialTheme.typography.bodyLarge,
						)
					},
					modifier = Modifier
						.fillMaxWidth()
						.height(40.dp.scaledHeight()),
				)
			},
			dismissButton = {
				TransparentButton(
					onClick = { pendingIcon = null },
					content = {
						Text(
							stringResource(R.string.cancel),
							style = MaterialTheme.typography.bodyLarge,
							color = MaterialTheme.colorScheme.onPrimaryContainer,
						)
					},
					modifier = Modifier
						.fillMaxWidth()
						.height(40.dp.scaledHeight()),
				)
			},
		)
	}
}
