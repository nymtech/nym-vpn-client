package net.nymtech.nymvpn.ui.screens.splittunneling.viewmodel

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.SplitTunnelingRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.ui.screens.splittunneling.model.AppFilter
import net.nymtech.nymvpn.ui.screens.splittunneling.model.AppInfo
import net.nymtech.nymvpn.ui.screens.splittunneling.model.SplitTunnelingUiState
import net.nymtech.nymvpn.ui.screens.splittunneling.model.UiEvent
import net.nymtech.vpn.backend.Tunnel
import javax.inject.Inject

@HiltViewModel
class SplitTunnelingViewModel @Inject constructor(
	@param:ApplicationContext private val context: Context,
	private val splitTunnelingRepository: SplitTunnelingRepository,
	private val backendManager: BackendManager,
	private val settingsRepository: SettingsRepository,
	private val helper: SplitTunnelingHelper,
) : ViewModel() {
	private val packageManager = context.packageManager
	private val _uiState = MutableStateFlow(SplitTunnelingUiState())
	val uiState = _uiState.asStateFlow()
	private var initialAppInfoList: List<AppInfo> = emptyList()

	private fun getInitData() {
		getAllInstalledAppList()
		onPerAppSecurityBannerDisplayed()
	}

	fun onUiEvent(event: UiEvent) {
		when (event) {
			is UiEvent.QueryChange -> filterApps(event.query)
			UiEvent.SelectAllDirectAppsClick -> filterAllDirectApps()
			UiEvent.SelectAllVpnPassThroughClick -> filterAllVpnPassThroughApps()
			is UiEvent.ChangeSelection -> changeChoiceSelection(event.packageName)
			is UiEvent.OnBackClick -> onBackClick(event.tunnelState)
			is UiEvent.ClearNavigation -> _uiState.update { it.copy(pendingNavigation = null) }
			is UiEvent.ClearDialog -> _uiState.update { it.copy(pendingDialog = null) }
			is UiEvent.NavigateBack -> _uiState.update { it.copy(pendingNavigation = SplitTunnelingUiState.PendingNavigation.NavigateBack) }
			UiEvent.LoadData -> getInitData()
		}
	}

	fun disconnect() {
		viewModelScope.launch {
			backendManager.stopTunnel()
			_uiState.update { it.copy(pendingDialog = null, pendingNavigation = SplitTunnelingUiState.PendingNavigation.NavigateToHome) }
		}
	}

	private fun onPerAppSecurityBannerDisplayed() = viewModelScope.launch {
		settingsRepository.setIsStreamServerBannerDisplayed(true)
	}

	private fun getAllInstalledAppList() {
		viewModelScope.launch(Dispatchers.Default) {
			_uiState.update { it.copy(isLoading = true) }
			runCatching {
				val (sortedSystemApps, sortedNormalApps) = helper.getInstalledApp(packageManager, splitTunnelingRepository)
				initialAppInfoList = sortedSystemApps + sortedNormalApps
				_uiState.update {
					it.copy(
						isLoading = false,
						systemApps = sortedSystemApps,
						normalApps = sortedNormalApps,
						filteredSystemApps = sortedSystemApps,
						filteredNormalApps = sortedNormalApps,
						directAppsCount = sortedSystemApps.totalAppCounts(false) + sortedNormalApps.totalAppCounts(false),
						vpnPassThroughAppsCount = sortedSystemApps.totalAppCounts(true) + sortedNormalApps.totalAppCounts(true),
					)
				}
			}.onFailure { _uiState.update { it.copy(isLoading = false) } }
		}
	}

	private fun filterApps(query: String) {
		viewModelScope.launch {
			_uiState.update {
				val (queryFilteredSystemApps, queryFilteredNormalApps) = helper.filterApps(query, it.systemApps, it.normalApps)
				val filteredSystemApps = queryFilteredSystemApps.filter { app -> it.appliedFilter == AppFilter.None || app.passThroughVpn == (it.appliedFilter == AppFilter.VpnPassThrough) }
				val filteredNormalApps = queryFilteredNormalApps.filter { app -> it.appliedFilter == AppFilter.None || app.passThroughVpn == (it.appliedFilter == AppFilter.VpnPassThrough) }
				val directAppsCount = if (it.appliedFilter == AppFilter.VpnPassThrough) {
					queryFilteredSystemApps.totalAppCounts(false) + queryFilteredNormalApps.totalAppCounts(false)
				} else {
					filteredSystemApps.totalAppCounts(false) + filteredNormalApps.totalAppCounts(false)
				}
				val vpnPassThroughAppsCount = if (it.appliedFilter == AppFilter.Direct) {
					queryFilteredSystemApps.totalAppCounts(true) + queryFilteredNormalApps.totalAppCounts(true)
				} else {
					filteredSystemApps.totalAppCounts(true) + filteredNormalApps.totalAppCounts(true)
				}
				it.copy(
					query = query,
					filteredSystemApps = filteredSystemApps,
					filteredNormalApps = filteredNormalApps,
					directAppsCount = directAppsCount,
					vpnPassThroughAppsCount = vpnPassThroughAppsCount,
				)
			}
		}
	}

	private fun filterAllDirectApps() {
		viewModelScope.launch {
			_uiState.update {
				val (filteredSystemApps, filteredNormalApps) = helper.filterDirectApps(it.appliedFilter, it.systemApps, it.normalApps)
				val isAlreadySelected = it.appliedFilter == AppFilter.Direct
				it.copy(
					filteredSystemApps = filteredSystemApps,
					filteredNormalApps = filteredNormalApps,
					directAppsCount = filteredSystemApps.totalAppCounts(false) + filteredNormalApps.totalAppCounts(false),
					vpnPassThroughAppsCount = it.systemApps.totalAppCounts(true) + it.normalApps.totalAppCounts(true),
					appliedFilter = if (!isAlreadySelected) AppFilter.Direct else AppFilter.None,
				)
			}
			if (uiState.value.query.isNotEmpty()) filterApps(uiState.value.query)
		}
	}

	private fun filterAllVpnPassThroughApps() {
		viewModelScope.launch {
			_uiState.update {
				val isAlreadySelected = it.appliedFilter == AppFilter.VpnPassThrough
				val filteredSystemApps = if (isAlreadySelected) it.systemApps else it.systemApps.filterAllPassThroughValue(true)
				val filteredNormalApps = if (isAlreadySelected) it.normalApps else it.normalApps.filterAllPassThroughValue(true)
				it.copy(
					filteredSystemApps = filteredSystemApps,
					filteredNormalApps = filteredNormalApps,
					directAppsCount = it.systemApps.totalAppCounts(false) + it.normalApps.totalAppCounts(false),
					vpnPassThroughAppsCount = filteredSystemApps.totalAppCounts(true) + filteredNormalApps.totalAppCounts(true),
					appliedFilter = if (!isAlreadySelected) AppFilter.VpnPassThrough else AppFilter.None,
				)
			}
			if (uiState.value.query.isNotEmpty()) filterApps(uiState.value.query)
		}
	}

	private fun changeChoiceSelection(packageName: String) {
		viewModelScope.launch {
			val updatedSystemApps = _uiState.value.systemApps.updatePassThroughValue(packageName)
			val updatedNormalApps = _uiState.value.normalApps.updatePassThroughValue(packageName)
			withContext(Dispatchers.IO) {
				splitTunnelingRepository.saveAppInfoList(updatedSystemApps + updatedNormalApps)
			}
			_uiState.update {
				val filteredSystemApps = it.filteredSystemApps.updatePassThroughValue(
					packageName,
				).filter { app -> it.appliedFilter == AppFilter.None || app.passThroughVpn == (it.appliedFilter == AppFilter.VpnPassThrough) }
				val filteredNormalApps = it.filteredNormalApps.updatePassThroughValue(
					packageName,
				).filter { app -> it.appliedFilter == AppFilter.None || app.passThroughVpn == (it.appliedFilter == AppFilter.VpnPassThrough) }
				val directAppsCount = if (it.appliedFilter == AppFilter.VpnPassThrough) {
					updatedSystemApps.totalAppCounts(false) + updatedNormalApps.totalAppCounts(false)
				} else {
					filteredSystemApps.totalAppCounts(false) + filteredNormalApps.totalAppCounts(false)
				}
				val vpnPassThroughAppsCount = if (it.appliedFilter == AppFilter.Direct) {
					updatedSystemApps.totalAppCounts(true) + updatedNormalApps.totalAppCounts(true)
				} else {
					filteredSystemApps.totalAppCounts(true) + filteredNormalApps.totalAppCounts(true)
				}

				it.copy(
					systemApps = updatedSystemApps,
					normalApps = updatedNormalApps,
					filteredSystemApps = filteredSystemApps,
					filteredNormalApps = filteredNormalApps,
					directAppsCount = directAppsCount,
					vpnPassThroughAppsCount = vpnPassThroughAppsCount,
				)
			}
		}
	}

	private fun onBackClick(tunnelState: Tunnel.State) {
		_uiState.update { it.copy(pendingNavigation = SplitTunnelingUiState.PendingNavigation.NavigateBack) }
		/*if (tunnelState != Tunnel.State.Up) {
			_uiState.update { it.copy(pendingNavigation = SplitTunnelingUiState.PendingNavigation.NavigateBack) }
			Timber.d("onBackClick: NavigateBack ${_uiState.value}")
		} else {
			if (initialAppInfoList != uiState.value.systemApps + uiState.value.normalApps) {
				_uiState.update { it.copy(pendingDialog = SplitTunnelingUiState.PendingDialog.AppListChangeDialog) }
			} else {
				_uiState.update { it.copy(pendingNavigation = SplitTunnelingUiState.PendingNavigation.NavigateBack) }
			}
		}*/
	}
}
