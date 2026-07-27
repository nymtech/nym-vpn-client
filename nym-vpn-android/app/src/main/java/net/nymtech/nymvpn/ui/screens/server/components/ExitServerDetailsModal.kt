package net.nymtech.nymvpn.ui.screens.server.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.LocationOn
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
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.Constants.URL_GATEWAYS_LOCATION
import net.nymtech.nymvpn.util.Constants.URL_STREAMING_SERVICES_ARTICLE
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
internal fun ExitServerDetailsModal(showModal: Boolean, onClick: (url: String) -> Unit, onDismiss: () -> Unit) {
	Modal(
		show = showModal,
		onDismiss = onDismiss,
		title = {
			Text(
				text = stringResource(R.string.exit_gateway_locations_title),
				style = CustomTypography.titleMediumPlus,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		text = {
			Column {
				Row(
					modifier = Modifier.fillMaxWidth(),
					horizontalArrangement = Arrangement.spacedBy(4.dp),
					verticalAlignment = Alignment.CenterVertically,
				) {
					Icon(
						ImageVector.vectorResource(R.drawable.smart_display),
						stringResource(R.string.stream_display),
						Modifier
							.size(20.dp)
							.align(Alignment.CenterVertically),
						tint = MaterialTheme.colorScheme.onSurface,
					)
					Text(
						text = stringResource(id = R.string.exit_gateway_locations_sub_item_1),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onSurface,
					)
				}
				Text(
					text = buildAnnotatedString {
						withStyle(
							style = SpanStyle(
								color = MaterialTheme.colorScheme.onBackground,
								textDecoration = TextDecoration.Underline,
							),
						) {
							withLink(
								LinkAnnotation.Url(url = URL_STREAMING_SERVICES_ARTICLE) {
									val url = (it as LinkAnnotation.Url).url
									onClick(url)
								},
							) {
								append(stringResource(R.string.exit_gateway_locations_sub_item_1_desc_link_text))
							}
						}

						append(" ")

						withStyle(
							style = SpanStyle(
								color = MaterialTheme.colorScheme.outline,
								fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
							),
						) {
							append(stringResource(R.string.exit_gateway_locations_sub_item_1_desc))
						}
					},
					modifier = Modifier.padding(top = 8.dp),
					style = MaterialTheme.typography.bodyMedium,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)

				Row(
					modifier = Modifier
						.fillMaxWidth()
						.padding(top = 16.dp),
					horizontalArrangement = Arrangement.spacedBy(4.dp),
					verticalAlignment = Alignment.CenterVertically,
				) {
					Icon(
						Icons.Outlined.LocationOn,
						stringResource(R.string.exit_gateway_locations_sub_item_2),
						Modifier
							.size(20.dp)
							.align(Alignment.CenterVertically),
						tint = MaterialTheme.colorScheme.onSurface,
					)
					Text(
						stringResource(id = R.string.exit_gateway_locations_sub_item_2),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onSurface,
					)
				}
				Text(
					text = buildAnnotatedString {
						withStyle(
							style = SpanStyle(
								color = MaterialTheme.colorScheme.outline,
								fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
							),
						) {
							append(stringResource(R.string.exit_gateway_locations_sub_item_2_desc))
						}

						append(" ")

						withStyle(
							style = SpanStyle(
								color = MaterialTheme.colorScheme.onBackground,
								textDecoration = TextDecoration.Underline,
							),
						) {
							withLink(
								LinkAnnotation.Url(url = URL_GATEWAYS_LOCATION) {
									val url = (it as LinkAnnotation.Url).url
									onClick(url)
								},
							) {
								append(stringResource(R.string.exit_gateway_locations_sub_item_2_desc_link_text))
							}
						}

						append(" ")

						withStyle(
							style = SpanStyle(
								color = MaterialTheme.colorScheme.outline,
								fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
							),
						) {
							append(stringResource(R.string.exit_gateway_locations_sub_item_2_desc_after_link))
						}
					},
					modifier = Modifier.padding(top = 8.dp),
					style = MaterialTheme.typography.bodyMedium,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)
			}
		},
		confirmButton = {
			MainStyledButton(
				onClick = onDismiss,
				textColor = Color.Black,
				content = { Text(stringResource(R.string.ok), fontFamily = FontFamily(Font(R.font.lab_grotesque_regular))) },
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
	)
}

@Composable
@PreviewLightDark
private fun ExitServerDetailsModalPreview() {
	NymVPNTheme(Theme.default()) {
		ExitServerDetailsModal(true, {}, {})
	}
}
