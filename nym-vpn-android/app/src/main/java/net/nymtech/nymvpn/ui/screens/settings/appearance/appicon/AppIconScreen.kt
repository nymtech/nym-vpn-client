package net.nymtech.nymvpn.ui.screens.settings.appearance.appicon

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Palette
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
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
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.core.graphics.drawable.toBitmap
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.domain.AppIcon
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.IconSurfaceButton
import net.nymtech.nymvpn.util.AppIconUtil
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel

@Composable
fun AppIconScreen(appUiState: AppUiState, viewModel: AppIconViewModel = hiltViewModel()) {
    val currentIcon by viewModel.currentIcon.collectAsState()
    val context = LocalContext.current
    var pendingIcon by remember { mutableStateOf<AppIcon?>(null) }

    // Switching the launcher icon kills the process via exitProcess, which
    // would tear down an in-process VPN service without a clean disconnect.
    // Gate the picker on Tunnel.State.Down — the dialog therefore never has
    // to warn about the VPN going away, because it can't.
    val canSwitch = appUiState.managerState.tunnelState == Tunnel.State.Down

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
                val label = stringResource(icon.labelRes)
                // Mipmaps for app icons are <adaptive-icon> XML which
                // Compose's painterResource() refuses to load. Inflate the
                // drawable and rasterize to a Bitmap for the preview.
                val previewBitmap = remember(icon) {
                    val drawable = ContextCompat.getDrawable(context, icon.previewDrawable)
                    drawable?.toBitmap(width = 144, height = 144)?.asImageBitmap()
                }
                IconSurfaceButton(
                    title = label,
                    onClick = { if (canSwitch) pendingIcon = icon },
                    selected = currentIcon == icon,
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
            title = { Text(stringResource(R.string.app_icon_change_title)) },
            text = { Text(stringResource(R.string.app_icon_change_body)) },
            confirmButton = {
                TextButton(
                    onClick = {
                        pendingIcon = null
                        AppIconUtil.apply(context, target)
                    },
                ) {
                    Text(
                        stringResource(R.string.app_icon_change_confirm),
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            },
            dismissButton = {
                TextButton(onClick = { pendingIcon = null }) {
                    Text(
                        stringResource(R.string.app_icon_change_cancel),
                        color = MaterialTheme.colorScheme.onBackground,
                    )
                }
            },
        )
    }
}
