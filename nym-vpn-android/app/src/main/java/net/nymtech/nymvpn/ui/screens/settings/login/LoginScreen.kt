package net.nymtech.nymvpn.ui.screens.settings.login

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.functions.rememberImeState
import net.nymtech.nymvpn.util.Event
import net.nymtech.nymvpn.util.Result
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun LoginScreen(
    navController: NavController,
    appViewModel: AppViewModel,
    viewModel: LoginViewModel = hiltViewModel()
) {

    val context = LocalContext.current

    var recoveryPhrase by remember {
        mutableStateOf("")
    }

    var error by remember {
        mutableStateOf<Event.Error>(Event.Error.None)
    }

    val imeState = rememberImeState()
    val scrollState = rememberScrollState()

    LaunchedEffect(imeState.value) {
        if (imeState.value) {
            scrollState.animateScrollTo(scrollState.viewportSize)
        }
    }


    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(40.dp.scaledHeight(), Alignment.Bottom),
        modifier = Modifier
            .fillMaxSize()
            .imePadding()
            .verticalScroll(scrollState)
            .padding(horizontal = 24.dp.scaledWidth())
    ) {
        Image(
            painter = painterResource(id = R.drawable.login),
            contentDescription = stringResource(id = R.string.login),
            contentScale = ContentScale.None,
            modifier = Modifier
                .padding(5.dp.scaledHeight())
                .width(120.dp)
                .height(120.dp)
        )
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Top), modifier = Modifier
                .padding(
                    horizontal = 24.dp.scaledWidth(), vertical = 24.dp.scaledHeight()
                )
        ) {
            Text(
                text = stringResource(id = R.string.welcome),
                style = MaterialTheme.typography.headlineSmall,
                color = MaterialTheme.colorScheme.onBackground
            )
            Text(
                text = stringResource(id = R.string.credential_message),
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center
            )
            Text(
                text = stringResource(id = R.string.credential_disclaimer),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center
            )
        }
        val isLoginError = error is Event.Error.LoginFailed
        OutlinedTextField(
            value = recoveryPhrase,
            onValueChange = {
                if (isLoginError) error = Event.Error.None
                recoveryPhrase = it
            },
            label = { Text(text = stringResource(id = R.string.credential_label)) },
            minLines = 3,
            maxLines = 3,
            isError = isLoginError,
            supportingText = {
                if (isLoginError) {
                    Text(
                        modifier = Modifier.fillMaxWidth(),
                        text = error.message.asString(context),
                        color = MaterialTheme.colorScheme.error
                    )
                }
            },
            modifier = Modifier
                .width(342.dp.scaledWidth())
                .height(196.dp.scaledHeight())
        )
        Box(
            modifier = Modifier
                .padding(bottom = 24.dp.scaledHeight())
        ) {
            MainStyledButton(
                onClick = {
                    viewModel.onLogin(recoveryPhrase).let {
                        when (it) {
                            is Result.Success -> {
                                navController.navigate(NavItem.Main.route)
                                appViewModel.showSnackbarMessage(context.getString(R.string.credential_successful))
                            }

                            is Result.Error -> error = it.error
                        }
                    }
                },
                content = {
                    Text(
                        stringResource(id = R.string.add_credential),
                        style = MaterialTheme.typography.labelLarge
                    )
                },
                color = MaterialTheme.colorScheme.primary
            )
        }
    }
}
