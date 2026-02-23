public extension HomeViewModel {
    @MainActor func connectDisconnect() async {
        guard !isConnectDisconnectInFlight else { return }
        isConnectDisconnectInFlight = true
        defer { isConnectDisconnectInFlight = false }

        guard connectionManager.currentTunnelStatus != .disconnecting
        else {
            return
        }

#if os(iOS)
        impactGenerator.impact()
        if connectionManager.currentTunnelStatus == .disconnected, !networkMonitor.isAvailable {
            Task { @MainActor in
                isOfflineOverlayDisplayed = true
            }
            return
        }
#endif
        resetStatusInfoState()

#if os(macOS)
        guard connectionManager.isMockModeEnabled || grpcManager.isServing
        else {
            navigateToDaemonEnable()
            return
        }
#endif

        // TODO: move to connection manager, do not check is valid imported if .connected
        if lastTunnelStatus != .connected {
            guard credentialsManager.isValidCredentialImported
            else {
                navigateToOnboarding()
                return
            }
            if await !credentialsManager.isAccountValid() {
                await credentialsManager.updateAccountSummary()
                if !credentialsManager.isAccountActive() {
                    navigateToPlanPurchase()
                    return
                }
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
        navigateToPlanPurchaseIfNeeded()
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

    func navigateToPlanPurchaseIfNeeded() {
        guard isLastErrorSubscriptionExpired() else { return }
        switch connectionManager.currentTunnelStatus {
        case .error:
            navigateToPlanPurchase()
        default:
            break
        }
    }
}
