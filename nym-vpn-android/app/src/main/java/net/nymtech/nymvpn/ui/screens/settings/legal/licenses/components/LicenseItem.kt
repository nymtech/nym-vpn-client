package net.nymtech.nymvpn.ui.screens.settings.legal.licenses.components

import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.screens.settings.legal.licenses.Artifact
import net.nymtech.nymvpn.ui.theme.iconSize

@Composable
fun LicenseItem(artifact: Artifact, onClick: () -> Unit) {
	SettingsGroup(
		items = listOf(
			SelectionItem(
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						artifact.name ?: stringResource(R.string.unknown),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						maxLines = 1,
						overflow = TextOverflow.Ellipsis,
					)
				},
				description = {
					Text(
						text = buildLicenseText(artifact),
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
					)
				},
				onClick = onClick,
			),
		),
	)
}

private fun buildLicenseText(artifact: Artifact): String {
	val spdxName = artifact.spdxLicenses?.map { it.name }
	val unknownNames = artifact.unknownLicenses?.map { it.name }
	val allNames = spdxName.orEmpty() + unknownNames.orEmpty()
	return allNames.distinct().joinToString()
}
