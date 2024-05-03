package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import okio.buffer
import okio.source
import javax.inject.Inject

@HiltViewModel
class LicensesViewModel
@Inject
constructor() : ViewModel() {
	private val _licences = MutableStateFlow<List<Artifact>>(emptyList())
	val licenses = _licences.asStateFlow()

	fun loadLicensesFromAssets(context: Context) = viewModelScope.launch {
		val source = context.assets.open("artifacts.json").source().buffer()
		_licences.update {
			LicenseParser.decode(source)
		}
		source.close()
	}
}
