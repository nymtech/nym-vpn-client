import Combine
import Foundation
import SwiftUI
import SnackbarManager
import AccountPrefetchGates
import AppSettings
import ConnectionManager
import ConnectionTypes
import CredentialsManager
import ErrorReason
import GatewayManager
import ImpactGenerator
import NetworkMonitor
import TunnelStatus
import UIComponents
#if os(macOS)
import GRPCManager
#endif

extension OneClickViewModel {
    func seedFromCurrentValues() {
        recomputeConnectState()
        refreshSelection()
    }

    func observe() {
        observeConnection()
        observeAccountAndEnvironment()
    }

    func observeConnection() {
        connectionManager.$currentTunnelStatus
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] status in
                guard let self else { return }
                if case .connected = status {
                    impactGenerator.success()
                }
                recomputeConnectState()
                refreshSelection()
            }
            .store(in: &cancellables)

        connectionManager.$lastError
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                self?.recomputeConnectState()
            }
            .store(in: &cancellables)

        connectionManager.$connectionInfoData
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                self?.refreshSelection()
            }
            .store(in: &cancellables)

        connectionManager.$connectionConfig
            .receive(on: DispatchQueue.main)
            .sink { [weak self] config in
                guard let self else { return }
                speedMode = OneClickSpeedMode(isTwoHop: config.enableTwoHop)
                refreshSelection()
            }
            .store(in: &cancellables)

#if SANTA
        Publishers.MergeMany(
            gatewayManager.$entry.map { _ in () },
            gatewayManager.$exit.map { _ in () },
            gatewayManager.$vpn.map { _ in () }
        )
        .receive(on: DispatchQueue.main)
        .sink { [weak self] in
            self?.refreshSelection()
        }
        .store(in: &cancellables)
#endif
    }

    func observeAccountAndEnvironment() {
        credentialsManager.$accountSummary
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                self?.recomputeConnectState()
            }
            .store(in: &cancellables)

        credentialsManager.$accountSummaryLastFetchFailed
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                self?.recomputeConnectState()
            }
            .store(in: &cancellables)

        appSettings.$isCredentialImportedPublisher
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                self?.recomputeConnectState()
            }
            .store(in: &cancellables)

        networkMonitor.$isAvailable
            .removeDuplicates()
            .dropFirst()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] isAvailable in
                guard let self else { return }
                recomputeConnectState()
                if isAvailable {
                    snackbarManager.clear()
                } else {
                    presentOfflineAlert()
                }
            }
            .store(in: &cancellables)

#if os(macOS)
        grpcManager.$isServing
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                self?.recomputeConnectState()
            }
            .store(in: &cancellables)
#endif
    }

    func recomputeConnectState() {
        let next = derivedConnectState()
        guard next != connectState else { return }
        connectState = next
    }

    func derivedConnectState() -> OneClickConnectState {
        switch connectionManager.currentTunnelStatus {
        case .connected:
            return .connected
        case .disconnecting:
            return .disconnecting
        case .error:
            return .stop
        case .offline, .offlineReconnect:
            return .noInternet
        case .connecting, .reasserting, .restarting:
            return .connecting
        case .disconnected, .unknown:
            if !networkMonitor.isAvailable {
                return .noInternet
            }
            return .disconnected(
                DisconnectedHomeCTA.resolve(
                    isCredentialImported: credentialsManager.isValidCredentialImported,
                    accountSummaryLastFetchFailed: credentialsManager.accountSummaryLastFetchFailed,
                    isAccountActive: credentialsManager.isAccountActive(),
                    hasAccountSummary: credentialsManager.accountSummary != nil
                )
            )
        }
    }

    func performConnectDisconnect(isConnectingTap: Bool) async {
        isConnectDisconnectInFlight = true
        defer { isConnectDisconnectInFlight = false }

        if isConnectingTap {
            let canProceed = await passesConnectPreflight()
            guard canProceed else { return }
        }

        do {
            try await connectionManager.connectDisconnect()
        } catch {
            impactGenerator.error()
            presentConnectionErrorAlert(
                message: ConnectionStatusViewModel.userFacingMessage(from: error)
            )
        }
        handleInactiveSubscriptionErrorIfNeeded()
        clearLastErrorIfNeeded()
    }

    func passesConnectPreflight() async -> Bool {
#if os(iOS)
        if !networkMonitor.isAvailable {
            presentOfflineAlert()
            return false
        }
#endif
#if os(macOS)
        if !connectionManager.isMockModeEnabled, !grpcManager.isServing {
            onRequestDaemonEnable?()
            return false
        }
#endif
        guard credentialsManager.isValidCredentialImported else { return false }
        guard await !credentialsManager.isAccountValid() else { return true }

        await credentialsManager.updateAccountSummary()
        let summary = credentialsManager.accountSummary
        let shouldOfferPurchase = ConnectPlanPurchaseGatePolicy.shouldOfferPlanPurchaseOnConnect(
            isAccountRegistrationInFlight: credentialsManager.isAccountRegistrationInFlight,
            accountSummaryLastFetchFailed: credentialsManager.accountSummaryLastFetchFailed,
            isAccountActive: credentialsManager.isAccountActive(),
            validUntilIsFuture: LoginSessionPolicy.validUntilIsFuture(
                validUntil: summary?.validUntilDate
            ),
            hasAccountSummary: summary != nil
        )
        if shouldOfferPurchase {
            sessionCoordinator?.handle(.requestInactiveSubscriptionPurchase)
            return false
        }
        return false
    }

    func handleInactiveSubscriptionErrorIfNeeded() {
        guard connectionManager.currentTunnelStatus == .error else { return }
        guard let error = connectionManager.lastError else { return }
        let reason: ErrorReason?
        if let typed = error as? ErrorReason {
            reason = typed
        } else {
            let nsError = error as NSError
            reason = nsError.domain == ErrorReason.domain ? ErrorReason(nsError: nsError) : nil
        }
        guard reason == .inactiveSubscription || reason == .inactiveAccount else { return }
        sessionCoordinator?.handle(.requestInactiveSubscriptionPurchase)
    }

    func clearLastErrorIfNeeded() {
        if isAwaitingGatewayIndependenceConsent {
            return
        }
        switch connectionManager.currentTunnelStatus {
        case .disconnecting, .disconnected, .error:
            connectionManager.lastError = nil
        default:
            break
        }
    }

    var isAwaitingGatewayIndependenceConsent: Bool {
        GatewayIndependenceArcPolicy.shouldPreserveIndependenceConsentError(
            status: connectionManager.currentTunnelStatus,
            lastError: connectionManager.lastError
        )
    }

    func applyDisplayMode(_ mode: OneClickDisplayMode) {
        displayMode = mode
        appSettings.oneClickDisplayModeRaw = mode.rawValue
        refreshSelection()
    }

    func presentOfflineAlert() {
        let tunnelIsUp: Bool
        switch connectionManager.currentTunnelStatus {
        case .offline, .offlineReconnect:
            tunnelIsUp = true
        default:
            tunnelIsUp = false
        }

        if tunnelIsUp {
            snackbarManager.enqueue(
                SnackbarItem(
                    style: .critical,
                    title: "offline".localizedString,
                    message: "connectionError.killswitchHint".localizedString,
                    actionTitle: "disconnect".localizedString,
                    onAction: { [weak self] in self?.disconnectFromOffline() },
                    duration: nil
                )
            )
        } else {
            snackbarManager.enqueue(
                SnackbarItem(
                    style: .critical,
                    title: "offline".localizedString,
                    message: "home.deviceNoInternet".localizedString
                )
            )
        }
    }

    func presentConnectionErrorAlert(message: String) {
        snackbarManager.enqueue(
            SnackbarItem(
                style: .critical,
                title: "connectionError.title".localizedString,
                message: ConnectionErrorCopy.message(reason: message),
                actionTitle: "disconnect".localizedString,
                onAction: { [weak self] in self?.disconnectFromError() },
                duration: 7
            )
        )
    }
}
