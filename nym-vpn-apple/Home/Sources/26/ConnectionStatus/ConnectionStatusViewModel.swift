import Combine
import Foundation
import ConnectionManager
import ErrorReason
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

    var hasFailure = false
    var lastConnectingStep: TunnelConnectingState?
    var lastDisplayedStep: ArcProgressState.Step?
    var lastErrorMessage: String?
    var mode: ArcProgressMode = .fast
    var status: TunnelStatus = .unknown
    var connectedDate: Date?

    @ObservationIgnored private var lastErrorSignature: ErrorSignature?
    @ObservationIgnored private var didFireForCurrentError = false

    /// Invoked when the tunnel reports `.error`. Parent (AppFeatureViewModel)
    /// uses this to surface a snackbar.
    @ObservationIgnored public var onConnectionFailed: ((String?) -> Void)?
    /// Invoked when a fresh connect attempt begins after the tunnel was idle.
    /// Parent uses this to clear stale failure snackbars.
    @ObservationIgnored public var onConnectionStarted: (() -> Void)?

    @ObservationIgnored private var cancellables = Set<AnyCancellable>()
    @ObservationIgnored private var queueTask: Task<Void, Never>?

    public init(connectionManager: ConnectionManager) {
        self.connectionManager = connectionManager
        seedFromCurrentValues()
        observe()
    }

    deinit {
        queueTask?.cancel()
    }

    var arcProgressState: ArcProgressState {
        switch status {
        case .connected:
            return .connected
        case .error:
            return .failed
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
}

private extension ConnectionStatusViewModel {
    func seedFromCurrentValues() {
        status = connectionManager.currentTunnelStatus
        lastConnectingStep = connectionManager.tunnelConnectingState
        connectedDate = connectionManager.connectedDate
        let error = connectionManager.lastError
        hasFailure = error != nil
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
            cancelQueue()

        case .error:
            cancelQueue()
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
            cancelQueue()

        case .connecting, .reasserting, .restarting, .offlineReconnect:
            if !wasConnecting {
                lastDisplayedStep = nil
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
        hasFailure = true
        lastErrorMessage = Self.userFacingMessage(from: error)
        fireConnectionFailedIfReady()
    }

    func fireConnectionFailedIfReady() {
        guard status == .error, !didFireForCurrentError, let lastErrorMessage else { return }
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

/// Maps daemon-reported `TunnelConnectingState` to the visual step shown by
/// `ArcProgressView`. Falls back to `.initializingNym` when the daemon hasn't
/// reported a step yet (cold connect) or reports `.unrecognized`.
private func arcStep(from step: TunnelConnectingState?) -> ArcProgressState.Step {
    switch step {
    case .resolvingApiAddresses:
        return .initializingNym
    case .awaitingAccountReadiness:
        return .authenticatingAccount
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
