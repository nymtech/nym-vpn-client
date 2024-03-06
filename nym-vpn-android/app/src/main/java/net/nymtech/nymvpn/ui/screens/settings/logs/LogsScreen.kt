package net.nymtech.nymvpn.ui.screens.settings.logs

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun LogsScreen(appViewModel: AppViewModel) {

    val logs = remember {
        appViewModel.logs
    }

    val lazyColumnListState = rememberLazyListState()
    val scope = rememberCoroutineScope()


    LaunchedEffect(logs.size){
        scope.launch {
            lazyColumnListState.animateScrollToItem(logs.size)
        }
    }

    LazyColumn(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Top),
        state = lazyColumnListState,
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 24.dp.scaledWidth())) {
        items(logs) {
            Row(horizontalArrangement = Arrangement.spacedBy(5.dp, Alignment.Start), verticalAlignment = Alignment.Top, modifier = Modifier.fillMaxSize()) {
                Text(text = it.tag, modifier = Modifier.fillMaxSize(0.3f))
                Text(text = it.level.signifier, modifier = Modifier.background(color = Color(it.level.color())).width(IntrinsicSize.Min))
                Text(it.message, color = MaterialTheme.colorScheme.onBackground)
            }
            
        }
    }
}
