package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun RadioSurfaceButton(leadingIcon: ImageVector? = null, title : String, description: String? = null, onClick: () -> Unit, selected: Boolean) {
    val border: BorderStroke? = if(selected) BorderStroke(1.dp,MaterialTheme.colorScheme.primary) else null
    val interactionSource = remember { MutableInteractionSource() }
    Card(
        modifier = Modifier.fillMaxWidth().height(64.dp.scaledHeight())
            .clickable(interactionSource = interactionSource, indication = null) {
                onClick()
            },
        border = border,
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)) {
        Column(
            modifier =
            Modifier.padding(end = 2.dp).fillMaxSize(), verticalArrangement = Arrangement.Center, horizontalAlignment = Alignment.Start) {
            Row(
                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp.scaledWidth())) {
                RadioButton(selected = selected, onClick = {onClick()}, modifier = Modifier.size(
                    iconSize).padding(start = 16.dp.scaledWidth()))
                Row(horizontalArrangement = Arrangement.spacedBy(16.dp.scaledHeight()), verticalAlignment = Alignment.CenterVertically) {
                    leadingIcon?.let {
                        Icon(leadingIcon, leadingIcon.name, Modifier.size(iconSize))
                    }
                    Column {
                        Text(title, style = MaterialTheme.typography.bodyLarge)
                        description?.let {
                            Text(
                                description,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                style = MaterialTheme.typography.bodyMedium)
                        }
                    }
                }
            }
        }
    }
}