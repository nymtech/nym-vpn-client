package net.nymtech.nymvpn.ui.screens.settings.appearance.appicon

import android.content.Context
import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.nymtech.nymvpn.data.domain.AppIcon
import net.nymtech.nymvpn.util.AppIconUtil
import javax.inject.Inject

@HiltViewModel
class AppIconViewModel
@Inject
constructor(
    @ApplicationContext private val context: Context,
) : ViewModel() {

    // The system (PackageManager) is the source of truth for the active alias;
    // we read it once on screen entry. AppIconUtil.apply() exits the process,
    // so there is no in-session "after apply" state to refresh.
    private val _currentIcon = MutableStateFlow(AppIconUtil.getCurrent(context))
    val currentIcon: StateFlow<AppIcon> = _currentIcon.asStateFlow()
}
