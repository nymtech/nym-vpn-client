package net.nymtech.nymvpn.ui.screens.settings.logs.modal

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.Share
import androidx.compose.material.icons.outlined.SaveAlt
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R

@Composable
fun LogsActionsMenu(onDownload: () -> Unit, onShare: () -> Unit, onDelete: () -> Unit) {
	var expanded by remember { mutableStateOf(false) }

	IconButton(onClick = { expanded = true }) {
		Icon(Icons.Filled.MoreVert, contentDescription = "More")
	}

	DropdownMenu(
		expanded = expanded,
		onDismissRequest = { expanded = false },
	) {
		DropdownMenuItem(
			text = { Text(stringResource(R.string.logs_download)) },
			leadingIcon = { Icon(Icons.Outlined.SaveAlt, contentDescription = null) },
			onClick = {
				expanded = false
				onDownload()
			},
		)
		DropdownMenuItem(
			text = { Text(stringResource(R.string.logs_share)) },
			leadingIcon = { Icon(Icons.Outlined.Share, contentDescription = null) },
			onClick = {
				expanded = false
				onShare()
			},
		)
		DropdownMenuItem(
			text = { Text(stringResource(R.string.logs_delete)) },
			leadingIcon = { Icon(Icons.Outlined.Delete, contentDescription = null) },
			onClick = {
				expanded = false
				onDelete()
			},
		)
	}
}
