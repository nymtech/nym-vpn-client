public extension HomeViewModel {
    func connectDisconnect() {
        guard connectionManager.currentTunnelStatus != .disconnecting
        else {
            return
        }

#if os(iOS)
        impactGenerator.impact()

        if !networkMonitor.isAvailable && connectionManager.currentTunnelStatus == .disconnected {
            Task { @MainActor in
                isOfflineOverlayDisplayed = true
            }
            return
        }
#endif
        Task {
            lastError = nil
            resetStatusInfoState()
#if os(macOS)
            guard helperInstallManager.daemonState != .installing else { return }
            do {
                try await helperInstallManager.installIfNeeded()
            } catch {
                updateStatusInfoState(with: .error(message: error.localizedDescription))
                updateConnectButtonState(with: .connect)
                return
            }

            updateStatusInfoState(with: .unknown)
            updateConnectButtonState(with: .connect)
#endif
            guard credentialsManager.isValidCredentialImported
            else {
                await navigateToAddCredentials()
                return
            }

            do {
                try await connectionManager.connectDisconnect()
            } catch let error {
                updateStatusInfoState(with: .error(message: error.localizedDescription))
#if os(iOS)
                impactGenerator.error()
#endif
            }
        }
    }
}
