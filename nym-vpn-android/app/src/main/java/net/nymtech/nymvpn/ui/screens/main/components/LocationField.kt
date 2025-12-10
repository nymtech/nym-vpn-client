package net.nymtech.nymvpn.ui.screens.main.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.ripple
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.screens.hop.components.QuickLabel
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.getFlagImageVectorByName
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun LocationField(
	value: String,
	label: String,
	countryCode: String?,
	gatewayLocation: String?,
	onClick: () -> Unit,
	onTrailingClick: () -> Unit,
	enabled: Boolean,
	modifier: Modifier = Modifier,
	showGatewayStreamIcon: Boolean = false,
	showQuicLabel: Boolean = false,
	showTrailingIcon: Boolean = true,
) {
	val context = LocalContext.current
	val trailingIcon = ImageVector.vectorResource(R.drawable.link_arrow_right)
	val indication = if (enabled) ripple() else null

	CustomTextField(
		value = value,
		readOnly = true,
		enabled = false,
		label = { Text(label, style = MaterialTheme.typography.bodySmall) },
		leading = {
			val (image, description) = countryCode?.let {
				Pair(
					ImageVector.vectorResource(context.getFlagImageVectorByName(it)),
					stringResource(R.string.country_flag, it),
				)
			} ?: Pair(
				ImageVector.vectorResource(R.drawable.faq),
				stringResource(R.string.unknown),
			)

			Image(
				image,
				description,
				modifier = Modifier
					.padding(
						horizontal = 16.dp.scaledWidth(),
						vertical = 16.dp.scaledHeight(),
					)
					.size(iconSize),
			)
		},
		trailing = {
			if (showGatewayStreamIcon || showQuicLabel || showTrailingIcon) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier.then(
						if (showGatewayStreamIcon || showQuicLabel) {
							Modifier.padding(horizontal = 12.dp)
						} else {
							Modifier
						},
					),
				) {
					if (showGatewayStreamIcon) {
						Icon(
							imageVector = ImageVector.vectorResource(R.drawable.smart_display),
							contentDescription = stringResource(R.string.stream_display),
							modifier = Modifier.size(iconSize),
							tint = Color.Unspecified,
						)
						Spacer(modifier = Modifier.width(4.dp))
					} else if (showQuicLabel) {
						QuickLabel()
					}

					if (showTrailingIcon) {
						Icon(
							imageVector = trailingIcon,
							contentDescription = stringResource(R.string.go),
							tint = MaterialTheme.colorScheme.onSurface,
							modifier = Modifier
								.size(iconSize)
								.clickable(
									interactionSource = remember { MutableInteractionSource() },
									indication = indication,
									enabled = enabled,
								) {
									onTrailingClick()
								},
						)
					}
				}
			}
		},
		singleLine = true,
		subtitle = {
			if (!gatewayLocation.isNullOrEmpty()) {
				Text(
					text = gatewayLocation,
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.outline,
					maxLines = 1,
					overflow = TextOverflow.Ellipsis,
				)
			}
		},
		modifier = modifier
			.fillMaxWidth()
			.height(60.dp.scaledHeight())
			.defaultMinSize(minHeight = 1.dp, minWidth = 1.dp)
			.clickable(
				interactionSource = remember { MutableInteractionSource() },
				indication = indication,
			) {
				if (enabled) onClick()
			},
	)
}
