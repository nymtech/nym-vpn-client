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
            guard !helperManager.isInstallNeeded()
            else {
                await navigateToInstallHelper()
                return
            }
#endif
            // TODO: move to connection manager, do not check is valid imported if .connected
            guard credentialsManager.isValidCredentialImported || lastTunnelStatus != .connected
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
