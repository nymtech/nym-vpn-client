package net.nymtech.nymvpn.ui.screens.account.passphrase.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R

@Composable
fun PassphraseCard(passphrase: List<String>, modifier: Modifier = Modifier, show: Boolean = false, onShowClick: () -> Unit = {}) {
	val shape = RoundedCornerShape(18.dp)
	Box(
		modifier
			.fillMaxWidth()
			.padding(top = 24.dp)
			.height(326.dp)
			.shadow(elevation = 6.dp, shape = shape, clip = false)
			.clip(shape)
			.background(MaterialTheme.colorScheme.primaryContainer)
			.border(BorderStroke(1.dp, MaterialTheme.colorScheme.outline), shape),
	) {
		if (!show) {
			OutlinedButton(
				onClick = {
					onShowClick()
				},
				border = BorderStroke(1.dp, MaterialTheme.colorScheme.onBackground.copy(alpha = 0.55f)),
				shape = RoundedCornerShape(24.dp),
				colors = ButtonDefaults.outlinedButtonColors(
					containerColor = Color.Transparent,
					contentColor = MaterialTheme.colorScheme.onBackground,
				),
				modifier = Modifier
					.align(Alignment.Center)
					.height(44.dp)
					.padding(horizontal = 24.dp),
			) {
				Icon(
					imageVector = Icons.Outlined.Visibility,
					contentDescription = "Show",
					modifier = Modifier.size(18.dp),
				)
				Spacer(Modifier.width(10.dp))
				Text(
					text = stringResource(R.string.passphrase_show),
					style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.SemiBold),
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
			}
		} else {
			Box(
				modifier = Modifier
					.fillMaxSize()
					.padding(horizontal = 16.dp),
			) {
				LazyVerticalGrid(
					columns = GridCells.Fixed(3),
					horizontalArrangement = Arrangement.spacedBy(16.dp),
					verticalArrangement = Arrangement.spacedBy(18.dp),
					modifier = Modifier
						.align(Alignment.Center)
						.padding(vertical = 24.dp),
				) {
					itemsIndexed(passphrase) { index, word ->
						Row(
							horizontalArrangement = Arrangement.spacedBy(4.dp, Alignment.Start),
							verticalAlignment = Alignment.CenterVertically,
						) {
							Text(
								text = "${index + 1}.",
								style = MaterialTheme.typography.bodyMedium,
								color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.75f),
							)
							Text(
								text = word,
								style = MaterialTheme.typography.bodyMedium,
								color = MaterialTheme.colorScheme.onPrimaryContainer,
								maxLines = 1,
								overflow = TextOverflow.Ellipsis,
							)
						}
					}
				}
			}
		}
	}
}
