package net.nymtech.nymvpn.ui.screens.settings.tunneling.components

import android.graphics.Bitmap
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
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.core.graphics.drawable.toBitmapOrNull
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.screens.settings.tunneling.AppInfo
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun AppInfoRow(appInfo: AppInfo, onTogglePassThrough: (String) -> Unit, mutableInteraction: MutableInteractionSource = MutableInteractionSource()) {
	val scheme = MaterialTheme.colorScheme
	Row(
		modifier = Modifier.fillMaxWidth(),
		verticalAlignment = Alignment.CenterVertically,
	) {
		loadIcon(appInfo.packageName)?.let {
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
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			modifier = Modifier.weight(1f),
		)
		Row(
			modifier = Modifier.clickable(
				indication = null,
				interactionSource = mutableInteraction,
				onClick = { onTogglePassThrough(appInfo.packageName) },
			),
			verticalAlignment = Alignment.CenterVertically,
		) {
			Box(
				modifier = Modifier
					.size(50.dp.scaledHeight(), 24.dp.scaledHeight())
					.background(
						color = if (!appInfo.passThroughVpn) scheme.errorContainer else scheme.background,
						shape = RoundedCornerShape(24.dp.scaledHeight()),
					),
				contentAlignment = Alignment.Center,
			) {
				Icon(
					painter = painterResource(R.drawable.split),
					contentDescription = null,
					modifier = Modifier.size(16.dp.scaledHeight()),
					tint = Color.White,
				)
			}
			Spacer(modifier = Modifier.width(8.dp.scaledWidth()))
			Box(
				modifier = Modifier
					.size(50.dp.scaledHeight(), 24.dp.scaledHeight())
					.background(
						color = if (appInfo.passThroughVpn) scheme.primaryContainer else scheme.background,
						shape = RoundedCornerShape(24.dp.scaledHeight()),
					),
				contentAlignment = Alignment.Center,
			) {
				Icon(
					imageVector = Icons.Filled.Shield,
					contentDescription = null,
					modifier = Modifier.size(16.dp.scaledHeight()),
					tint = if (appInfo.passThroughVpn) scheme.primary else scheme.onPrimaryContainer,
				)
			}
		}
	}
}

@Composable
fun loadIcon(packageName: String): Bitmap? {
	val context = LocalContext.current
	val packageManager = remember(context) { context.packageManager }
	return try {
		packageManager.getApplicationIcon(packageName).toBitmapOrNull()
	} catch (_: Exception) {
		null
	}
}
