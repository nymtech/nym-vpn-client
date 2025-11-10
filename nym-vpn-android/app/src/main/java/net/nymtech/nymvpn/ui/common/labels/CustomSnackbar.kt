package net.nymtech.nymvpn.ui.common.labels

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarData
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.common.snackbar.CustomSnackbarContent
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun CustomSnackBar(
	data: SnackbarData,
	paddingTop: Dp,
	isRtl: Boolean = false,
	containerColor: Color = CustomColors.snackBarBackgroundColor,
	content: MutableState<CustomSnackbarContent?>,
) {
	if (content.value != null) {
		content.value?.let { info ->
			Box(
				modifier = Modifier
					.fillMaxSize()
					.padding(horizontal = 24.dp.scaledWidth())
					.padding(top = paddingTop),
				contentAlignment = Alignment.TopCenter,
			) {
				Snackbar(containerColor = containerColor) {
					CompositionLocalProvider(
						LocalLayoutDirection provides if (isRtl) LayoutDirection.Rtl else LayoutDirection.Ltr,
					) {
						Row(
							modifier = Modifier
								.fillMaxWidth()
								.padding(horizontal = 4.dp, vertical = 4.dp),
							verticalAlignment = Alignment.CenterVertically,
						) {
							Text(
								text = info.message,
								color = CustomColors.snackbarTextColor,
								modifier = Modifier
									.weight(1f)
									.padding(end = 8.dp),
							)
							Row(
								verticalAlignment = Alignment.CenterVertically,
								horizontalArrangement = Arrangement.End,
							) {
								info.action?.title?.let { title ->
									TextButton(onClick = { info.action?.onActionPress?.invoke() }) {
										Text(text = title, style = MaterialTheme.typography.labelLarge, maxLines = 1)
									}
								}
								info.iconAction?.let {
									Box(
										modifier = Modifier
											.size(24.dp)
											.clickable {
												data.dismiss()
												it.onActionPress()
											},
										contentAlignment = Alignment.Center,
									) {
										Icon(
											imageVector = it.icon,
											contentDescription = "Dismiss",
											modifier = Modifier.size(24.dp),
											tint = Color.White,
										)
									}
								}
							}
						}
					}
				}
			}
		}
	} else {
		Box(
			modifier = Modifier
				.fillMaxSize()
				.padding(horizontal = 24.dp.scaledWidth())
				.padding(top = paddingTop),
			contentAlignment = Alignment.TopCenter,
		) {
			Snackbar(containerColor = containerColor) {
				CompositionLocalProvider(
					LocalLayoutDirection provides if (isRtl) LayoutDirection.Rtl else LayoutDirection.Ltr,
				) {
					Row(verticalAlignment = Alignment.Top, horizontalArrangement = Arrangement.Center) {
						Text(data.visuals.message, color = CustomColors.snackbarTextColor)
						data.visuals.actionLabel?.let {
							Box(
								Modifier.clickable {
									data.performAction()
								},
							) {
								Text(it, color = CustomColors.snackbarTextColor)
							}
						}
					}
				}
			}
		}
	}
}
