package net.nymtech.nymvpn.ui.common

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.CustomColors

@Composable
internal fun InfoBanner(showBanner: Boolean, config: BannerConfig, modifier: Modifier = Modifier) {
	AnimatedVisibility(showBanner) {
		Row(
			modifier = modifier
				.fillMaxWidth()
				.clip(RoundedCornerShape(8.dp))
				.background(CustomColors.snackBarBackgroundColor)
				.padding(horizontal = 16.dp, vertical = 10.dp),
			verticalAlignment = Alignment.CenterVertically,
		) {
			Text(
				text = config.message,
				color = CustomColors.snackbarTextColor,
				modifier = Modifier
					.weight(1f)
					.padding(end = 8.dp),
			)
			Row(
				verticalAlignment = Alignment.CenterVertically,
				horizontalArrangement = Arrangement.End,
			) {
				config.action?.let { action ->
					TextButton(onClick = { action.onClicked() }, modifier = Modifier.padding(end = 12.dp)) {
						Text(text = action.title, style = MaterialTheme.typography.labelLarge, maxLines = 2)
					}
				}
				config.icon?.let {
					Box(
						modifier = Modifier
							.size(24.dp)
							.clickable {
								it.onClicked()
							},
						contentAlignment = Alignment.Center,
					) {
						Icon(
							imageVector = it.icon,
							contentDescription = stringResource(R.string.close),
							modifier = Modifier.size(24.dp),
							tint = Color.White,
						)
					}
				}
			}
		}
	}
}

@Immutable
data class BannerConfig(
	val message: String,
	val action: BannerAction?,
	val icon: BannerIcon?,
)

@Immutable
data class BannerAction(
	val title: String,
	val onClicked: () -> Unit,
)

@Immutable
data class BannerIcon(
	val icon: ImageVector,
	val onClicked: () -> Unit,
)
