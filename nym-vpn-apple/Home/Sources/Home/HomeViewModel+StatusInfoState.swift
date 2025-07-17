import Foundation
import SwiftUI
import ErrorReason
import UIComponents

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

    @MainActor func displayUnsecureBannerIfNeeded(with error: Error?) {
        guard let errorReason = error as? ErrorReason,
              errorReason == .subscriptionExpired
        else {
            return
        }
        isBannerDisplayed = true
    }

    @MainActor func updateLastError(_ error: Error?) {
        if lastError == nil, let error {
            updateStatusInfoState(with: .error(message: error.localizedDescription))
            navigateToAddCredetialsIfNeeded(error: error)
            displayUnsecureBannerIfNeeded(with: error)
            lastError = error
        } else {
            guard let lastNsError = lastError as? NSError,
                  let nsError = error as? NSError,
                  lastNsError.domain != nsError.domain,
                  lastNsError.code != nsError.code
            else {
                return
            }

            updateStatusInfoState(with: .error(message: nsError.localizedDescription))
            navigateToAddCredetialsIfNeeded(error: error)
            displayUnsecureBannerIfNeeded(with: error)
            lastError = error
        }
    }

    @MainActor func resetBannerDisplay() {
        isBannerDisplayed = false
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
}
