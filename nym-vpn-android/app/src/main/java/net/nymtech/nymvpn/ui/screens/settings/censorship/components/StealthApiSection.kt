package net.nymtech.nymvpn.ui.screens.settings.censorship.components

import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.openWebUrl

@Composable
fun StealthApiSection(shape: Shape = RoundedCornerShape(8.dp), modifier: Modifier = Modifier) {
	val context = LocalContext.current
	val interactionSource = remember { MutableInteractionSource() }

	Card(
		modifier = modifier,
		shape = shape,
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
	) {
		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(horizontal = 16.dp, vertical = 16.dp),
		) {
			Row(
				modifier = Modifier.fillMaxWidth(),
				horizontalArrangement = Arrangement.SpaceBetween,
				verticalAlignment = Alignment.CenterVertically,
			) {
				Text(
					text = stringResource(R.string.censorship_stealth_title),
					style = MaterialTheme.typography.titleMedium,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)
				ScaledSwitch(
					checked = true,
					enabled = false,
					onClick = {},
				)
			}
			Text(
				text = stringResource(R.string.censorship_stealth_description),
				style = Typography.bodySmall,
				color = MaterialTheme.colorScheme.outline,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				modifier = Modifier
					.fillMaxWidth()
					.padding(top = 16.dp),
				textAlign = TextAlign.Justify,
			)

			Box(
				contentAlignment = Alignment.Center,
				modifier = Modifier
					.fillMaxWidth()
					.padding(top = 16.dp)
					.clickable(
						interactionSource = interactionSource,
						indication = null,
					) {
						context.openWebUrl(context.getString(R.string.censorship_stealth_link))
					},
			) {
				Text(
					text = stringResource(R.string.censorship_stealth_link_text),
					style = MaterialTheme.typography.bodyMedium.copy(
						textDecoration = TextDecoration.Underline,
					),
					modifier = Modifier.fillMaxWidth(),
				)
			}
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewStealthApiSection() {
	NymVPNTheme(Theme.default()) {
		StealthApiSection(modifier = Modifier.fillMaxWidth())
	}
}
