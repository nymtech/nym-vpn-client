package net.nymtech.nymvpn.ui.screens.settings.tunneling.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Shield
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.screens.settings.tunneling.AppInfo
import net.nymtech.nymvpn.ui.screens.settings.tunneling.UiEvent
import net.nymtech.nymvpn.ui.theme.CustomColorsPalette
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun AppInfoRow(customColorPalette: CustomColorsPalette, appInfo: AppInfo, onUiEvent: (UiEvent) -> Unit, mutableInteraction: MutableInteractionSource = MutableInteractionSource()) {
	Row(
		modifier = Modifier.fillMaxWidth(),
		verticalAlignment = Alignment.CenterVertically,
	) {
		LoadIcon(appInfo.packageName)?.let {
			Icon(
				it.asImageBitmap(),
				contentDescription = appInfo.name,
				tint = Color.Unspecified,
				modifier = Modifier
					.padding(end = 16.dp.scaledHeight())
					.size(iconSize.scaledHeight()),
			)
		}
		Text(
			text = appInfo.name,
			style = Typography.bodyMedium,
			color = MaterialTheme.colorScheme.onBackground,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			modifier = Modifier.weight(1f),
		)
		Row(
			modifier = Modifier
				.clickable(
					indication = null,
					interactionSource = mutableInteraction,
					onClick = { onUiEvent(UiEvent.ChangeSelection(appInfo.packageName)) },
				),
			verticalAlignment = Alignment.CenterVertically,
		) {
			Box(
				modifier = Modifier
					.size(50.dp.scaledHeight(), 24.dp.scaledHeight())
					.background(
						color = if (!appInfo.passThroughVpn) customColorPalette.redIconBackground else customColorPalette.greyIconBackground,
						shape = RoundedCornerShape(24.dp.scaledHeight()),
					),
				contentAlignment = Alignment.Center,
			) {
				Icon(
					painterResource(R.drawable.split),
					contentDescription = null,
					modifier = Modifier
						.size(16.dp.scaledHeight()),
					tint = if (!appInfo.passThroughVpn) customColorPalette.redIcon else customColorPalette.greyIcon,
				)
			}
			Spacer(modifier = Modifier.width(8.dp.scaledWidth()))
			Box(
				modifier = Modifier
					.size(50.dp.scaledHeight(), 24.dp.scaledHeight())
					.background(
						color = if (appInfo.passThroughVpn) customColorPalette.greenIconBackground else customColorPalette.greyIconBackground,
						shape = RoundedCornerShape(24.dp.scaledHeight()),
					),
				contentAlignment = Alignment.Center,
			) {
				Icon(
					Icons.Filled.Shield,
					contentDescription = null,
					modifier = Modifier
						.size(16.dp.scaledHeight()),
					tint = if (appInfo.passThroughVpn) customColorPalette.greenIcon else customColorPalette.greyIcon,
				)
			}
		}
	}
}
