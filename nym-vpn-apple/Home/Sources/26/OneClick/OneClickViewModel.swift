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

@MainActor
@Observable
public final class OneClickViewModel {
    // Internal (not `private`) so the `+State` / `+Selection` extension files can reach them.
    let appSettings: AppSettings
    let connectionManager: ConnectionManager
    let credentialsManager: CredentialsManager
    let gatewayManager: GatewayManager
    let snackbarManager: SnackbarManager
    let impactGenerator: ImpactGenerator
    let networkMonitor: NetworkMonitor
#if os(macOS)
    let grpcManager: GRPCManager
#endif

    var connectState: OneClickConnectState = .disconnected
    var entrySelectionPhase: OneClickSelectionPhase = .selecting
    var selectionPhase: OneClickSelectionPhase = .selecting
    var isLiveConnection: Bool = false

    var displayMode: OneClickDisplayMode

    var speedMode: OneClickSpeedMode

    /// Invoked when the daemon reports `.inactiveSubscription` or when the
    /// pre-flight gate detects an expired account. Routes the user into the
    /// purchase flow.
    @ObservationIgnored public weak var sessionCoordinator: AppSessionCoordinating?
    /// macOS only: invoked when a connect attempt is made while the helper
    /// daemon is not running, so the user can install/enable it.
    @ObservationIgnored public var onRequestDaemonEnable: (() -> Void)?

    @ObservationIgnored var connectDisconnectTask: Task<Void, Never>?
    @ObservationIgnored var resolveTask: Task<Void, Never>?
    @ObservationIgnored var cancellables = Set<AnyCancellable>()
    @ObservationIgnored var isConnectDisconnectInFlight = false

#if os(iOS)
    public init(
        appSettings: AppSettings,
        connectionManager: ConnectionManager,
        credentialsManager: CredentialsManager,
        gatewayManager: GatewayManager,
        snackbarManager: SnackbarManager,
        impactGenerator: ImpactGenerator,
        networkMonitor: NetworkMonitor
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.credentialsManager = credentialsManager
        self.gatewayManager = gatewayManager
        self.snackbarManager = snackbarManager
        self.impactGenerator = impactGenerator
        self.networkMonitor = networkMonitor

        self.displayMode = OneClickDisplayMode(rawValue: appSettings.oneClickDisplayModeRaw) ?? .powerUser
        self.speedMode = OneClickSpeedMode(
            isTwoHop: connectionManager.connectionConfig.enableTwoHop
        )

        seedFromCurrentValues()
        observe()
    }
#elseif os(macOS)
    public init(
        appSettings: AppSettings,
        connectionManager: ConnectionManager,
        credentialsManager: CredentialsManager,
        gatewayManager: GatewayManager,
        snackbarManager: SnackbarManager,
        impactGenerator: ImpactGenerator,
        networkMonitor: NetworkMonitor,
        grpcManager: GRPCManager
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.credentialsManager = credentialsManager
        self.gatewayManager = gatewayManager
        self.snackbarManager = snackbarManager
        self.impactGenerator = impactGenerator
        self.networkMonitor = networkMonitor
        self.grpcManager = grpcManager

        self.displayMode = OneClickDisplayMode(rawValue: appSettings.oneClickDisplayModeRaw) ?? .powerUser
        self.speedMode = OneClickSpeedMode(
            isTwoHop: connectionManager.connectionConfig.enableTwoHop
        )

        seedFromCurrentValues()
        observe()
    }
#endif

    func connectButtonTapped() {
        guard !isConnectDisconnectInFlight else { return }
        guard connectionManager.currentTunnelStatus != .disconnecting else { return }

        impactGenerator.impact()
        snackbarManager.clear()

        if handleDisconnectedHomeCTATap() {
            return
        }

        let isConnectingTap = connectionManager.currentTunnelStatus != .connected

        connectDisconnectTask?.cancel()
        connectDisconnectTask = Task { @MainActor [weak self] in
            await self?.performConnectDisconnect(isConnectingTap: isConnectingTap)
        }
    }

    func handleDisconnectedHomeCTATap() -> Bool {
        switch connectState {
        case .noAccount:
            sessionCoordinator?.handle(.requestWelcome)
            return true
        case .noSubscription:
            sessionCoordinator?.handle(.requestInactiveSubscriptionPurchase)
            return true
        case .accountUnreachable:
            Task { @MainActor [weak self] in
                await self?.credentialsManager.updateAccountSummary(force: true)
            }
            return true
        case .disconnected, .connecting, .stop, .connected, .disconnecting, .noInternet:
            return false
        }
    }

    func disconnectFromError() {
        connectDisconnectTask?.cancel()
        connectDisconnectTask = Task { @MainActor [weak self] in
            guard let self else { return }
            snackbarManager.clear()
            isConnectDisconnectInFlight = true
            defer { isConnectDisconnectInFlight = false }
            do {
                try await connectionManager.connectDisconnect()
                connectionManager.lastError = nil
            } catch {
                impactGenerator.error()
            }
        }
    }

    func disconnectFromOffline() {
        connectDisconnectTask?.cancel()
        connectDisconnectTask = Task { @MainActor [weak self] in
            guard let self else { return }
            snackbarManager.clear()
            isConnectDisconnectInFlight = true
            defer { isConnectDisconnectInFlight = false }
            await connectionManager.disconnectAndWaitForDisconnected()
            connectionManager.lastError = nil
        }
    }

    func independenceConsentAgreed() {
        Task { @MainActor [weak self] in
            guard let self else { return }
            snackbarManager.clear()
            do {
                try await connectionManager.acceptRelaxedGatewayIndependence()
            } catch {
                impactGenerator.error()
                presentConnectionErrorAlert(
                    message: ConnectionStatusViewModel.userFacingMessage(from: error)
                )
            }
        }
    }

    func cancelGatewayIndependenceConsent() {
        guard !isConnectDisconnectInFlight,
              connectionManager.currentTunnelStatus == .error
        else {
            return
        }

        impactGenerator.impact()
        snackbarManager.clear()
    }

    func upCaretTapped() {
        guard displayMode == .powerUser else { return }
        impactGenerator.softImpact()
        applyDisplayMode(.nerd)
    }

    func downCaretTapped() {
        guard displayMode == .nerd else { return }
        impactGenerator.softImpact()
        applyDisplayMode(.powerUser)
    }

    func setSpeedMode(_ mode: OneClickSpeedMode) {
        guard mode != speedMode else { return }
        impactGenerator.softImpact()
        speedMode = mode

        let cfg = connectionManager.connectionConfig
        switch mode {
        case .fast:
            if !cfg.enableTwoHop {
                connectionManager.setTwoHop(true)
            }
        case .anonymous:
            if cfg.enableTwoHop {
                connectionManager.setTwoHop(false)
            }
        }
    }
}

extension OneClickSpeedMode {
    init(isTwoHop: Bool) {
        self = isTwoHop ? .fast : .anonymous
    }
}
