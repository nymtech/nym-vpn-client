package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import android.app.Application
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import okio.buffer
import okio.source
import javax.inject.Inject

@HiltViewModel
class LicensesViewModel @Inject constructor(
    private val application: Application,
) : ViewModel() {

    private val _licences = MutableStateFlow<List<Artifact>>(emptyList())
    val licenses = _licences.asStateFlow()
    fun loadLicensesFromAssets() =
        viewModelScope.launch {
            val source = application.assets.open("artifacts.json").source().buffer()
            _licences.value = LicenseParser.decode(source)
        }
}