package net.nymtech.nymvpn.ui.common

import androidx.compose.foundation.background
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.ShapeDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.iconSize

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SearchBar(onQuery: (queryString: String) -> Unit, placeholder : (@Composable () -> Unit)) {
    // Immediately update and keep track of query from text field changes.
    val space = " "
    var query: String by rememberSaveable { mutableStateOf("") }
    val interactionSource = remember { MutableInteractionSource() }

    val colors = OutlinedTextFieldDefaults.colors(
            focusedTextColor = MaterialTheme.colorScheme.onSurface,
            focusedBorderColor = MaterialTheme.colorScheme.outline,
            focusedLeadingIconColor = MaterialTheme.colorScheme.onSurface,
            focusedTrailingIconColor = MaterialTheme.colorScheme.onSurface,
            focusedLabelColor = MaterialTheme.colorScheme.onSurface,
            cursorColor = MaterialTheme.colorScheme.onSurface,
            focusedPlaceholderColor = MaterialTheme.colorScheme.onSurface,
            focusedSupportingTextColor = MaterialTheme.colorScheme.onSurface,
            unfocusedLabelColor = MaterialTheme.colorScheme.onSurface,
            focusedPrefixColor = MaterialTheme.colorScheme.onSurface,
            focusedSuffixColor = MaterialTheme.colorScheme.onSurface
        )
    BasicTextField(
        value = query,
        onValueChange = { onQueryChanged : String ->
            // If user makes changes to text, immediately updated it.
            query = onQueryChanged
            onQuery(onQueryChanged)
        },
        singleLine = true,
        cursorBrush = SolidColor(MaterialTheme.colorScheme.onBackground),
        modifier =
        Modifier
            .fillMaxWidth()
            .background(color = Color.Transparent, RoundedCornerShape(30.dp))
    ) { innerTextField ->
        OutlinedTextFieldDefaults.DecorationBox(
            value = space + query,
            leadingIcon = {
            val icon = Icons.Rounded.Search
            Icon(
                imageVector = icon,
                modifier = Modifier.size(iconSize),
                tint = MaterialTheme.colorScheme.onBackground,
                contentDescription = icon.name)
            },
            label = { Text(stringResource(R.string.search), modifier = Modifier.padding(start = 8.dp)) },
            singleLine = true,
            enabled = true,
            innerTextField = {
                if(query.isEmpty()) {
                    placeholder()
                }
                innerTextField.invoke()
             },
            visualTransformation = VisualTransformation.None,
            colors = colors,
            interactionSource = interactionSource,
            container = {
                OutlinedTextFieldDefaults.ContainerBox(enabled = true, isError = false, colors = colors, interactionSource = interactionSource, focusedBorderThickness = 1.dp, unfocusedBorderThickness = 1.dp, shape = ShapeDefaults.Small)
            },
        )
    }
}
