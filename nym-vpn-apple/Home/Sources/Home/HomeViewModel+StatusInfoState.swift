import Foundation
import SwiftUI
import Device
import ErrorReason
import MessageModels
import UIComponents
import TunnelStatus
#if os(macOS)
import NymVPNRpc
#endif

extension HomeViewModel {
    func updateConnectButtonStateIfMnemonicImported() {
        guard !appSettings.isCredentialImported
        else {
            if connectButtonState == .noAccount {
                connectButtonState = .connect
            }
            return
        }
        connectButtonState = .noAccount
    }

    @MainActor func resetLastError() {
        lastError = nil
    }

    @MainActor func resetStatusInfoState() {
        updateStatusInfoState(with: .unknown)
    }

    @MainActor func updateStatusInfoState(with newState: StatusInfoState) {
        guard newState != statusInfoState else { return }
        statusInfoState = newState
    }

    @MainActor func navigateToAddCredetialsIfNeeded(error: Error?) {
        guard let errorReason = error as? ErrorReason,
              errorReason == .noAccountStored
        else {
            return
        }
        resetStatusInfoState()
        navigateToAddCredentials()
    }

    func isLastErrorSubscriptionExpired() -> Bool {
        guard let errorReason = lastError as? ErrorReason,
              errorReason == .inactiveSubscription
        else {
            return false
        }
        return true
    }

    @MainActor func updateLastError(_ error: Error?) {
        if lastError == nil, let error {
            #if os(macOS)
            let errorDescription = if let rpcError = error as? RpcError {
                rpcError.message()
            } else {
                error.localizedDescription
            }
            #else
            let errorDescription = error.localizedDescription
            #endif
            updateStatusInfoState(with: .error(message: errorDescription))
            navigateToAddCredetialsIfNeeded(error: error)
            lastError = error
        } else {
            guard let lastNsError = lastError as? NSError,
                  let nsError = error as? NSError,
                  lastNsError.domain != nsError.domain,
                  lastNsError.code != nsError.code
            else {
                return
            }
            #if os(macOS)
            let errorDescription = if let rpcError = error as? RpcError {
                rpcError.message()
            } else {
                nsError.localizedDescription
            }
            #else
            let errorDescription = nsError.localizedDescription
            #endif

            updateStatusInfoState(with: .error(message: errorDescription))
            navigateToAddCredetialsIfNeeded(error: error)
            lastError = error
        }
    }

    func offlineState(with hasInternet: Bool) {
        Task { @MainActor [weak self] in
            guard let self else { return }
            withAnimation { [weak self] in
                guard let self else { return }
                statusButtonConfig = StatusButtonConfig(tunnelStatus: lastTunnelStatus, hasInternet: hasInternet)
                statusInfoState = StatusInfoState(hasInternet: hasInternet)
            }
        }
    }

    func displayEnableStatisticsSnackBarCTAIfNeeded() {
        guard lastTunnelStatus == .disconnected,
              !appSettings.isStatisticsEnabled,
              appSettings.statisticsConnectionCount == 1 || appSettings.statisticsConnectionCount.isMultiple(of: 10),
              Device.isMacOS
        else {
            return
        }
        messagesManager.addAndProcess(
            SnackBarMessage(
                text: "statisticsOverlay.snackbar.helpImprove".localizedString,
                style: .noIcon,
                ctaText: "statisticsOverlay.snackbar.improveNow".localizedString,
                ctaAction: { [weak self] in
                    self?.isStatisticsOverlayDisplayed = true
                }
            )
        )
    }
}
