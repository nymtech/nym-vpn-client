public extension HomeViewModel {
    @MainActor func connectDisconnect() async {
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
        guard grpcManager.isServing
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
