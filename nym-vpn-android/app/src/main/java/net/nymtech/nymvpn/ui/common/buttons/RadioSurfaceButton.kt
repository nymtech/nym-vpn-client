package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import androidx.window.core.layout.WindowHeightSizeClass
import net.nymtech.nymvpn.ui.MainActivity

@Composable
fun RadioSurfaceButton(leadingIcon: ImageVector? = null, title : String, description: String? = null, onClick: () -> Unit, selected: Boolean) {
    val border: BorderStroke? = if(selected) BorderStroke(1.dp,MaterialTheme.colorScheme.primary) else null
    val interactionSource = remember { MutableInteractionSource() }
    val cardHeight = when(MainActivity.windowHeightSizeClass) {
        WindowHeightSizeClass.MEDIUM, WindowHeightSizeClass.COMPACT -> 56.dp
        else -> { 64.dp }}
    val descriptionTypography = when(MainActivity.windowHeightSizeClass) {
        WindowHeightSizeClass.MEDIUM, WindowHeightSizeClass.COMPACT -> MaterialTheme.typography.bodySmall
        else -> { MaterialTheme.typography.bodyMedium }}
    val titleTypography = when(MainActivity.windowHeightSizeClass) {
        WindowHeightSizeClass.MEDIUM, WindowHeightSizeClass.COMPACT -> MaterialTheme.typography.titleMedium
        else -> { MaterialTheme.typography.bodyLarge }}
    Card(
        modifier = Modifier.fillMaxWidth().height(cardHeight)
            .clickable(interactionSource = interactionSource, indication = null) {
                onClick()
            },
        border = border,
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)) {
        Box(
            modifier =
            Modifier.padding(top = 8.dp, bottom = 8.dp, end = 2.dp)) {
            Row(
                verticalAlignment = Alignment.CenterVertically) {
                RadioButton(selected = selected, onClick = {onClick()})
                Row(horizontalArrangement = Arrangement.spacedBy(16.dp), verticalAlignment = Alignment.CenterVertically) {
                    leadingIcon?.let {
                        Icon(leadingIcon, leadingIcon.name)
                    }
                    Column {
                        Text(title, style = titleTypography)
                        description?.let {
                            Text(
                                description,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                style = descriptionTypography)
                        }
                    }
                }
            }
        }
    }
}