package net.nymtech.nymvpn.ui.screens.server.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material.icons.rounded.StarBorder
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
internal fun ServerDetailsTrailingContent(isFavorite: Boolean, onToggleFavorite: () -> Unit, onInfoIconClick: () -> Unit) {
	Box(
		modifier = Modifier.fillMaxHeight(),
		contentAlignment = Alignment.CenterEnd,
	) {
		Row(
			horizontalArrangement = Arrangement.spacedBy(8.dp.scaledWidth()),
			verticalAlignment = Alignment.CenterVertically,
			modifier = Modifier
				.padding(end = 16.dp.scaledWidth())
				.heightIn(min = 42.dp.scaledHeight())
				.align(Alignment.CenterEnd),
		) {
			Icon(
				imageVector = if (isFavorite) Icons.Rounded.Star else Icons.Rounded.StarBorder,
				contentDescription = "Favorite",
				tint = if (isFavorite) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onBackground,
				modifier = Modifier
					.size(iconSize.scaledHeight())
					.clickable { onToggleFavorite() },
			)
			Icon(
				imageVector = Icons.Outlined.Info,
				contentDescription = stringResource(R.string.info),
				modifier = Modifier
					.size(iconSize.scaledHeight())
					.clickable { onInfoIconClick() },
			)
		}
	}
}
