package net.nymtech.nymvpn.ui.screens.settings.tunneling

import android.content.Context
import android.os.Build
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.SplitTunnelingRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.ui.common.events.UiEvent
import net.nymtech.nymvpn.ui.common.events.UiEvent as CommonUiEvent
import net.nymtech.nymvpn.util.SplitTunnelingHelper
import net.nymtech.nymvpn.util.extensions.isVpnLockdownEnabled
import net.nymtech.nymvpn.util.filterAllPassThroughValue
import net.nymtech.nymvpn.util.totalAppCounts
import net.nymtech.nymvpn.util.updatePassThroughValue
import net.nymtech.vpn.backend.Tunnel
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class SplitTunnelingViewModel @Inject constructor(
	@param:ApplicationContext private val context: Context,
	private val splitTunnelingRepository: SplitTunnelingRepository,
	private val backendManager: BackendManager,
	private val settingsRepository: SettingsRepository,
	private val helper: SplitTunnelingHelper,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-split-tunnel-vm"
	}

	private val packageManager = context.packageManager

	private val _uiState = MutableStateFlow(SplitTunnelingUiState())
	val uiState = _uiState.asStateFlow()

	private val _backendUi = MutableStateFlow(SplitTunnelingBackendUiState())
	val backendUi = _backendUi.asStateFlow()

	private val _events = MutableSharedFlow<CommonUiEvent>(
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	val events = _events.asSharedFlow()

	private var initialAppInfoList: List<AppInfo> = emptyList()

	init {
		viewModelScope.launch {
			backendManager.stateFlow.collect { s ->
				_backendUi.value = SplitTunnelingBackendUiState(
					tunnelState = s.tunnelState,
					isRestarting = s.isRestarting,
				)
			}
		}
	}

	fun loadData() {
		getAllInstalledAppList()
		onPerAppSecurityBannerDisplayed()
		updateLockdownState()
	}

	fun onQueryChange(query: String) {
		filterApps(query)
	}

	fun onSelectAllDirectAppsClick() {
		filterAllDirectApps()
	}

	fun onSelectAllVpnPassThroughClick() {
		filterAllVpnPassThroughApps()
	}

	fun onChangeSelection(packageName: String) {
		changeChoiceSelection(packageName)
	}

	fun clearSaveDialog() {
		_uiState.update { it.copy(showSaveChangesDialog = false) }
	}

	fun requestBack() {
		if (_uiState.value.hasUnsavedChanges) {
			_uiState.update { it.copy(showSaveChangesDialog = true) }
		} else {
			_uiState.update { it.copy(navigateBack = true) }
		}
	}

	fun consumeNavigateBack() {
		_uiState.update { it.copy(navigateBack = false) }
	}

	fun discardAndNavigateBack() {
		_uiState.update { it.copy(showSaveChangesDialog = false, navigateBack = true) }
	}

	fun saveChangesAndMaybeReconnect(isActuallyConnected: Boolean) {
		viewModelScope.launch(Dispatchers.IO) {
			runCatching {
				val toSave = _uiState.value.systemApps + _uiState.value.normalApps
				splitTunnelingRepository.saveAppInfoList(toSave)

				initialAppInfoList = toSave

				_uiState.update {
					it.copy(
						hasUnsavedChanges = false,
						showSaveChangesDialog = false,
					)
				}

				Timber.tag(TAG).i(
					"SplitTunnelingSaved count=%d reconnect=%s",
					toSave.size,
					isActuallyConnected,
				)

				if (isActuallyConnected) {
					notifyReconnectIfConnected()
					backendManager.requestReconnect()
				}
			}.onFailure { t ->
				Timber.tag(TAG).e(t, "SplitTunnelingSaveFailed")
			}
		}
	}

	private fun notifyReconnectIfConnected() {
		val state = backendManager.getState()
		val isConnected = state == Tunnel.State.Up || state == Tunnel.State.EstablishingConnection

		if (isConnected) {
			_events.tryEmit(UiEvent.ReconnectStarted)
		}
	}

	private fun onPerAppSecurityBannerDisplayed() = viewModelScope.launch {
		settingsRepository.setIsStreamServerBannerDisplayed(true)
	}

	private fun updateLockdownState() {
		val lockdownState = when {
			!isVpnLockdownEnabled(context) -> LockdownState.OFF
			Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q -> LockdownState.ACTIVE_STEERING
			else -> LockdownState.UNSUPPORTED_API
		}
		_uiState.update { it.copy(lockdownState = lockdownState) }
	}

	private fun getAllInstalledAppList() {
		viewModelScope.launch(Dispatchers.Default) {
			_uiState.update { it.copy(isLoading = true) }

			runCatching {
				val (sortedSystemApps, sortedNormalApps) =
					helper.getInstalledApp(packageManager, splitTunnelingRepository)

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
						hasUnsavedChanges = false,
					)
				}

				Timber.tag(TAG).d(
					"SplitTunnelingAppsLoaded system=%d normal=%d",
					sortedSystemApps.size,
					sortedNormalApps.size,
				)
			}.onFailure { t ->
				Timber.tag(TAG).e(t, "SplitTunnelingAppsLoadFailed")
				_uiState.update { it.copy(isLoading = false) }
			}
		}
	}

	private fun filterApps(query: String) {
		viewModelScope.launch {
			_uiState.update {
				val (queryFilteredSystemApps, queryFilteredNormalApps) =
					helper.filterApps(query, it.systemApps, it.normalApps)

				val filteredSystemApps =
					queryFilteredSystemApps.filter { app ->
						it.appliedFilter == AppFilter.None ||
							app.passThroughVpn == (it.appliedFilter == AppFilter.VpnPassThrough)
					}

				val filteredNormalApps =
					queryFilteredNormalApps.filter { app ->
						it.appliedFilter == AppFilter.None ||
							app.passThroughVpn == (it.appliedFilter == AppFilter.VpnPassThrough)
					}

				val directAppsCount =
					if (it.appliedFilter == AppFilter.VpnPassThrough) {
						queryFilteredSystemApps.totalAppCounts(false) + queryFilteredNormalApps.totalAppCounts(false)
					} else {
						filteredSystemApps.totalAppCounts(false) + filteredNormalApps.totalAppCounts(false)
					}

				val vpnPassThroughAppsCount =
					if (it.appliedFilter == AppFilter.Direct) {
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
				val (filteredSystemApps, filteredNormalApps) =
					helper.filterDirectApps(it.appliedFilter, it.systemApps, it.normalApps)

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

			_uiState.update {
				val filteredSystemApps = it.filteredSystemApps.updatePassThroughValue(packageName)
					.filter { app ->
						it.appliedFilter == AppFilter.None ||
							app.passThroughVpn == (it.appliedFilter == AppFilter.VpnPassThrough)
					}

				val filteredNormalApps = it.filteredNormalApps.updatePassThroughValue(packageName)
					.filter { app ->
						it.appliedFilter == AppFilter.None ||
							app.passThroughVpn == (it.appliedFilter == AppFilter.VpnPassThrough)
					}

				val directAppsCount =
					if (it.appliedFilter == AppFilter.VpnPassThrough) {
						updatedSystemApps.totalAppCounts(false) + updatedNormalApps.totalAppCounts(false)
					} else {
						filteredSystemApps.totalAppCounts(false) + filteredNormalApps.totalAppCounts(false)
					}

				val vpnPassThroughAppsCount =
					if (it.appliedFilter == AppFilter.Direct) {
						updatedSystemApps.totalAppCounts(true) + updatedNormalApps.totalAppCounts(true)
					} else {
						filteredSystemApps.totalAppCounts(true) + filteredNormalApps.totalAppCounts(true)
					}

				val currentList = updatedSystemApps + updatedNormalApps
				val hasUnsavedChanges = currentList != initialAppInfoList

				it.copy(
					systemApps = updatedSystemApps,
					normalApps = updatedNormalApps,
					filteredSystemApps = filteredSystemApps,
					filteredNormalApps = filteredNormalApps,
					directAppsCount = directAppsCount,
					vpnPassThroughAppsCount = vpnPassThroughAppsCount,
					hasUnsavedChanges = hasUnsavedChanges,
				)
			}
		}
	}
}
