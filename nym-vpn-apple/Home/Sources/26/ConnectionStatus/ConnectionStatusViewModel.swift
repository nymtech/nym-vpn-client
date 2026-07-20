import Combine
import Foundation
import ConnectionManager
import ConnectionTypes
import ErrorReason
import NetworkMonitor
import Theme
import TunnelStatus
import UIComponents
#if os(iOS)
import ErrorHandler
import UIKit
#endif

@Observable
@MainActor public final class ConnectionStatusViewModel {
    public let connectionManager: ConnectionManager
    private let networkMonitor: NetworkMonitor

    var hasFailure = false
    var isOnline = true
    var lastConnectingStep: TunnelConnectingState?
    var lastDisplayedStep: ArcProgressState.Step?
    var lastErrorMessage: String?
    var mode: ArcProgressMode = .fast
    var status: TunnelStatus = .unknown
    var connectedDate: Date?
    var showsIndependenceWarning = false

    @ObservationIgnored private var lastErrorSignature: ErrorSignature?
    @ObservationIgnored private var didFireForCurrentError = false
    @ObservationIgnored private var resumeFromIndependenceConsent = false

    /// Invoked when the tunnel reports `.error`. Parent (AppFeatureViewModel)
    /// uses this to surface a snackbar.
    @ObservationIgnored public var onConnectionFailed: ((String?) -> Void)?
    /// Invoked when a fresh connect attempt begins after the tunnel was idle.
    /// Parent uses this to clear stale failure snackbars.
    @ObservationIgnored public var onConnectionStarted: (() -> Void)?

    @ObservationIgnored private var cancellables = Set<AnyCancellable>()
    @ObservationIgnored private var queueTask: Task<Void, Never>?

    public init(
        connectionManager: ConnectionManager,
        networkMonitor: NetworkMonitor = .shared
    ) {
        self.connectionManager = connectionManager
        self.networkMonitor = networkMonitor
        self.isOnline = networkMonitor.isAvailable
        seedFromCurrentValues()
        observe()
    }

    deinit {
        queueTask?.cancel()
    }

    var arcProgressState: ArcProgressState {
        if !isOnline {
            return .offline
        }
        switch status {
        case .connected:
            return .connected
        case .error:
            if GatewayIndependenceArcPolicy.shouldUseAwaitingGatewayConsentArc(
                status: status,
                lastError: connectionManager.lastError
            ) {
                return .awaitingGatewayConsent
            }
            if GatewayIndependenceArcPolicy.shouldUseFailedArc(
                status: status,
                lastError: connectionManager.lastError
            ) {
                return .failed
            }
            return .step(lastDisplayedStep ?? .choosingBestServers)
        case .connecting, .reasserting, .restarting, .offlineReconnect:
            return .step(lastDisplayedStep ?? .initializingNym)
        case .disconnecting:
            return .canceling
        case .disconnected, .offline, .unknown:
            return hasFailure ? .failed : .disconnected
        }
    }

    public func setMode(_ mode: ArcProgressMode) {
        self.mode = mode
    }

    public var isConnectingLike: Bool { status.isConnectingLike }
}

private extension ConnectionStatusViewModel {
    func seedFromCurrentValues() {
        status = connectionManager.currentTunnelStatus
        lastConnectingStep = connectionManager.tunnelConnectingState
        connectedDate = connectionManager.connectedDate
        mode = resolvedArcMode(
            liveTunnelType: connectionManager.connectionInfoData?.tunnelType,
            connectionType: connectionManager.connectionType
        )
        let error = connectionManager.lastError
        hasFailure = GatewayIndependenceArcPolicy.shouldRecordConnectionFailure(error)
        lastErrorMessage = error.map { Self.userFacingMessage(from: $0) }

        if status.isConnectingLike {
            lastDisplayedStep = arcStep(from: lastConnectingStep)
        } else if case .connected = status {
            lastDisplayedStep = .establishingConnection
        }
    }

    func observe() {
        connectionManager.$currentTunnelStatus
            .receive(on: DispatchQueue.main)
            .sink { [weak self] in self?.apply(status: $0) }
            .store(in: &cancellables)

        connectionManager.$tunnelConnectingState
            .receive(on: DispatchQueue.main)
            .sink { [weak self] in self?.apply(connectingStep: $0) }
            .store(in: &cancellables)

        connectionManager.$lastError
            .receive(on: DispatchQueue.main)
            .sink { [weak self] in self?.apply(error: $0) }
            .store(in: &cancellables)

        connectionManager.$connectedDate
            .receive(on: DispatchQueue.main)
            .sink { [weak self] in self?.connectedDate = $0 }
            .store(in: &cancellables)

        connectionManager.$connectionType
            .receive(on: DispatchQueue.main)
            .sink { [weak self] connectionType in
                guard let self else { return }
                self.mode = resolvedArcMode(
                    liveTunnelType: self.connectionManager.connectionInfoData?.tunnelType,
                    connectionType: connectionType
                )
            }
            .store(in: &cancellables)

        connectionManager.$connectionInfoData
            .receive(on: DispatchQueue.main)
            .sink { [weak self] info in
                guard let self else { return }
                self.mode = resolvedArcMode(
                    liveTunnelType: info?.tunnelType,
                    connectionType: self.connectionManager.connectionType
                )
            }
            .store(in: &cancellables)

        networkMonitor.$isAvailable
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] isAvailable in
                self?.isOnline = isAvailable
            }
            .store(in: &cancellables)
    }

    func apply(status newStatus: TunnelStatus) {
        let wasConnecting = status.isConnectingLike
        let statusChanged = status != newStatus

        if case .connecting = newStatus {
            hasFailure = false
        }
        if case .connected = newStatus {
            hasFailure = false
        }

        status = newStatus

        if statusChanged {
            announceStatusChange(newStatus)
        }

        switch newStatus {
        case .connected:
            lastDisplayedStep = .establishingConnection
            showsIndependenceWarning = false
            resumeFromIndependenceConsent = false
            lastErrorSignature = nil
            didFireForCurrentError = false
            cancelQueue()

        case .error:
            cancelQueue()
            if GatewayIndependenceArcPolicy.isIndependenceConsentError(connectionManager.lastError) {
                // Keep `lastDisplayedStep` (the reached ring) so the consent
                // arc holds it and the resume rolls forward — don't unload.
                resumeFromIndependenceConsent = true
            }
            fireConnectionFailedIfReady()

        case .disconnecting:
            hasFailure = false
            cancelQueue()

        case .disconnected, .offline, .unknown:
            hasFailure = false
            lastErrorSignature = nil
            didFireForCurrentError = false
            lastDisplayedStep = nil
            lastConnectingStep = nil
            resumeFromIndependenceConsent = false
            showsIndependenceWarning = false
            cancelQueue()

        case .connecting, .reasserting, .restarting, .offlineReconnect:
            if !wasConnecting {
                if resumeFromIndependenceConsent {
                    lastDisplayedStep = Self.resumedDisplayedStep(current: lastDisplayedStep)
                } else {
                    lastDisplayedStep = nil
                    showsIndependenceWarning = false
                }
                resumeFromIndependenceConsent = false
                onConnectionStarted?()
            }
            scheduleQueueIfNeeded()

        @unknown default:
            break
        }
    }

    func apply(connectingStep step: TunnelConnectingState?) {
        lastConnectingStep = step
        scheduleQueueIfNeeded()
    }

    func apply(error: Error?) {
        guard let error else {
            hasFailure = false
            lastErrorMessage = nil
            lastErrorSignature = nil
            didFireForCurrentError = false
            return
        }
        let signature = ErrorSignature(error: error)
        if signature == lastErrorSignature {
            return
        }
        lastErrorSignature = signature
        didFireForCurrentError = false
        hasFailure = GatewayIndependenceArcPolicy.shouldRecordConnectionFailure(error)
        lastErrorMessage = Self.userFacingMessage(from: error)
        if GatewayIndependenceArcPolicy.isIndependenceConsentError(error) {
            // Keep the reached ring so the consent arc holds it (no unload).
            resumeFromIndependenceConsent = true
        }
        fireConnectionFailedIfReady()
    }

    func fireConnectionFailedIfReady() {
        guard status == .error, !didFireForCurrentError, let lastErrorMessage else {
            return
        }
        didFireForCurrentError = true
        onConnectionFailed?(lastErrorMessage)
    }

    func scheduleQueueIfNeeded() {
        guard status.isConnectingLike else { return }
        let target = arcStep(from: lastConnectingStep)
        guard ordinal(of: lastDisplayedStep) < ordinal(of: target) else { return }
        guard queueTask == nil else { return }
        startQueueTick()
    }

    func startQueueTick() {
        let duration = mode.sweepDuration
        queueTask = Task { [weak self] in
            try? await Task.sleep(for: duration)
            guard !Task.isCancelled else { return }
            self?.advanceDisplayedStep()
        }
    }

    func advanceDisplayedStep() {
        queueTask = nil
        guard status.isConnectingLike else { return }
        let target = arcStep(from: lastConnectingStep)
        let nextOrdinal = ordinal(of: lastDisplayedStep) + 1
        let allCases = ArcProgressState.Step.allCases
        guard nextOrdinal < allCases.count, nextOrdinal <= ordinal(of: target)
        else {
            return
        }
        lastDisplayedStep = allCases[nextOrdinal]
        if ordinal(of: lastDisplayedStep) < ordinal(of: target) {
            startQueueTick()
        }
    }

    func cancelQueue() {
        queueTask?.cancel()
        queueTask = nil
    }

    func announceStatusChange(_ newStatus: TunnelStatus) {
#if os(iOS)
        guard UIAccessibility.isVoiceOverRunning,
              let key = accessibilityAnnouncementKey(for: newStatus) else { return }
        UIAccessibility.post(notification: .announcement, argument: key.localizedString)
#endif
    }

    func accessibilityAnnouncementKey(for status: TunnelStatus) -> String? {
        switch status {
        case .connected:
            return "accessibility.tunnelStatus.connected"
        case .connecting, .reasserting, .restarting:
            return "accessibility.tunnelStatus.connecting"
        case .disconnecting:
            return "accessibility.tunnelStatus.disconnecting"
        case .disconnected:
            return "accessibility.tunnelStatus.disconnected"
        case .offline, .offlineReconnect:
            return "accessibility.tunnelStatus.offline"
        case .error:
            return "accessibility.tunnelStatus.error"
        case .unknown:
            return nil
        @unknown default:
            return nil
        }
    }
}

extension ConnectionStatusViewModel {
    /// Surfaces the localized human-readable message for any error coming from
    /// `connectionManager.lastError`. Handles both already-typed reasons and
    /// raw NSErrors (which race through `tunnel.$lastError` before
    /// TunnelsManager performs the typed conversion). Mirrors the domain
    /// dispatch in `TunnelsManager.updateLastTunnelErrorIfNeeded`.
    public static func userFacingMessage(from error: Error) -> String {
        if let reason = error as? ErrorReason {
            return reason.errorDescription ?? error.localizedDescription
        }
#if os(iOS)
        if let reason = error as? VPNErrorReason {
            return reason.errorDescription ?? error.localizedDescription
        }
#endif
        let nsError = error as NSError
        switch nsError.domain {
        case ErrorReason.domain:
            if let reason = ErrorReason(nsError: nsError) {
                return reason.errorDescription ?? error.localizedDescription
            }
#if os(iOS)
        case VPNErrorReason.domain:
            if let reason = VPNErrorReason(nsError: nsError) {
                return reason.errorDescription ?? error.localizedDescription
            }
#endif
        default:
            break
        }
        return error.localizedDescription
    }

    public static func isNeedsRelaxedIndependenceCriteria(_ error: Error?) -> Bool {
        GatewayIndependenceArcPolicy.isIndependenceConsentError(error)
    }

    /// Displayed step when a connect resumes after gateway-independence
    /// consent. Keeps whatever ring the connect had already reached so the arc
    /// only rolls forward (macOS reaches the middle ring before the error);
    /// falls back to the outer ring when nothing was reached yet (iOS
    /// pre-flight errors before any step).
    static func resumedDisplayedStep(
        current: ArcProgressState.Step?
    ) -> ArcProgressState.Step? {
        current ?? .authenticatingAccount
    }
}

enum ConnectionErrorCopy {
    /// Snackbar body for connection failures: `<reason>` paragraph, then the
    /// killswitch hint, then the disconnect instruction on its own line. The
    /// reason is parsed via `ConnectionStatusViewModel.userFacingMessage`.
    static func message(reason: String?) -> String {
        let hint = "connectionError.killswitchHint".localizedString
        let instruction = "connectionError.disconnectInstruction".localizedString
        let tail = hint + "\n\n" + instruction
        guard let reason, !reason.isEmpty else { return tail }
        return reason + "\n\n" + tail
    }
}

private struct ErrorSignature: Equatable {
    let domain: String
    let code: Int

    init(error: Error) {
        let nsError = error as NSError
        self.domain = nsError.domain
        self.code = nsError.code
    }
}

private func ordinal(of step: ArcProgressState.Step?) -> Int {
    guard let step,
          let index = ArcProgressState.Step.allCases.firstIndex(of: step)
    else {
        return -1
    }
    return index
}

private extension TunnelStatus {
    /// `true` for any status the arc treats as "in progress".
    var isConnectingLike: Bool {
        switch self {
        case .connecting, .reasserting, .restarting, .offlineReconnect:
            return true
        default:
            return false
        }
    }
}

private func resolvedArcMode(
    liveTunnelType: ConnectionTunnelType?,
    connectionType: ConnectionType
) -> ArcProgressMode {
    if let liveTunnelType {
        switch liveTunnelType {
        case .wireguard:
            return .fast
        case .mixnet:
            return .anonymous
        }
    }
    switch connectionType {
    case .wireguard:
        return .fast
    case .mixnet5hop:
        return .anonymous
    }
}

/// Maps daemon-reported `TunnelConnectingState` to the visual step shown by
/// `ArcProgressView`. Falls back to `.initializingNym` when the daemon hasn't
/// reported a step yet (cold connect) or reports `.unrecognized`.
private func arcStep(from step: TunnelConnectingState?) -> ArcProgressState.Step {
    switch step {
    case .resolvingApiAddresses:
        return .initializingNym
    case .awaitingAccountReadiness:
        return .authenticatingAccount
    case .awaitingCredentialsAvailability:
        return .downloadingZkNyms
    case .refreshingGateways:
        return .updatingServerList
    case .selectingGateways:
        return .choosingBestServers
    case .registeringWithGateways:
        return .registeringWithServers
    case .connectingTunnel:
        return .establishingConnection
    case .unrecognized, .none:
        return .initializingNym
    }
}
