public extension HomeViewModel {
    func connectDisconnect() async {
        guard connectionManager.currentTunnelStatus != .disconnecting
        else {
            return
        }

#if os(iOS)
        impactGenerator.impact()
        if connectionManager.currentTunnelStatus == .disconnected {
            if !networkMonitor.isAvailable {
                Task { @MainActor in
                    isOfflineOverlayDisplayed = true
                }
                return
            } else {
                resetBannerDisplay()
            }
        }
#endif
        resetStatusInfoState()
#if os(macOS)
        guard !helperManager.isInstallNeeded()
        else {
            navigateToInstallHelper()
            return
        }
#endif
        // TODO: move to connection manager, do not check is valid imported if .connected
        if lastTunnelStatus != .connected {
            guard credentialsManager.isValidCredentialImported
            else {
                navigateToAddCredentials()
                return
            }
        }

        do {
            try await connectionManager.connectDisconnect()
        } catch let error {
            updateStatusInfoState(with: .error(message: error.localizedDescription))
#if os(iOS)
            impactGenerator.error()
#endif
        }
        clearLastErrorIfNeeded()
    }

    func clearLastErrorIfNeeded() {
        switch connectionManager.currentTunnelStatus {
        case .disconnecting, .disconnected, .error:
            resetLastError()
        default:
            break
        }
    }
}
