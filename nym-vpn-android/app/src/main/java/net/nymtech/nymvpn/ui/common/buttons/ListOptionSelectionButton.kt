package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.window.core.layout.WindowHeightSizeClass
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.MainActivity

@Composable
fun ListOptionSelectionButton(
    label: String,
    value: String,
    onClick : () -> Unit,
    leadingIcon: @Composable () -> Unit,
    trailingIcon: ImageVector = ImageVector.vectorResource(R.drawable.link_arrow_right)
) {
    val height = when(MainActivity.windowHeightSizeClass) {
        WindowHeightSizeClass.MEDIUM, WindowHeightSizeClass.COMPACT -> 60.dp
        else -> { 60.dp } }
    val interactionSource = remember { MutableInteractionSource() }
    OutlinedTextField(
        value = value,
        placeholder = { Text(value, style = MaterialTheme.typography.bodyLarge ,overflow = TextOverflow.Visible) },
        onValueChange = {},
        enabled = false,
        colors = OutlinedTextFieldDefaults.colors(
            disabledTextColor = MaterialTheme.colorScheme.onSurface,
            disabledContainerColor = Color.Transparent,
            disabledBorderColor = MaterialTheme.colorScheme.outline,
            disabledLeadingIconColor = MaterialTheme.colorScheme.onSurface,
            disabledTrailingIconColor = MaterialTheme.colorScheme.onSurface,
            disabledLabelColor = MaterialTheme.colorScheme.onSurfaceVariant,
            disabledPlaceholderColor = MaterialTheme.colorScheme.onSurface,
            disabledSupportingTextColor = MaterialTheme.colorScheme.onSurface,
            unfocusedLabelColor = MaterialTheme.colorScheme.onSurface,
            disabledPrefixColor = MaterialTheme.colorScheme.onSurface,
            disabledSuffixColor = MaterialTheme.colorScheme.onSurface
        ),
        modifier = Modifier.fillMaxWidth().height(height).clickable(interactionSource = interactionSource, indication = null){ onClick() },
        readOnly = true,

        label = {
            Text(label, style = MaterialTheme.typography.bodySmall) },
        leadingIcon = {
            leadingIcon()
        },
        trailingIcon = {
            Icon(trailingIcon, trailingIcon.name, tint = MaterialTheme.colorScheme.onSurface)
        })
}
