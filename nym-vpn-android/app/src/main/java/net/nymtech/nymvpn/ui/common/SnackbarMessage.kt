package net.nymtech.nymvpn.ui.common

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp

@Composable
fun SnackbarMessage(message : String, padding : Dp) {
    Text(
        message,
        Modifier.padding(horizontal = padding).padding(top = padding),
        textAlign = TextAlign.Center, color = MaterialTheme.colorScheme.secondary)
}