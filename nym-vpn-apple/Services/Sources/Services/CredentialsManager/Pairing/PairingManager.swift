#if os(iOS)
import Foundation
import Logging
import UIKit
import ErrorHandler
import NymVPNLib

@MainActor
public final class PairingManager: ObservableObject {
    public static let shared = PairingManager(credentialsManager: .shared)

    static let codeLifetime: Duration = .seconds(5 * 60)

    @Published public private(set) var state: PairingState = .idle

    private let credentialsManager: CredentialsManager
    private let logger = Logger(label: "PairingManager")
    private var queue: Task<Void, Never>?
    private var eventsTask: Task<Void, Never>?
    private var backgroundObserver: NSObjectProtocol?

    init(credentialsManager: CredentialsManager) {
        self.credentialsManager = credentialsManager
        // The core doesn't replay pairing events, so they're collected for the app's lifetime.
        eventsTask = Task { [weak self, events = credentialsManager.accountControllerEvents] in
            for await event in events {
                guard case let .pairing(pairingEvent) = event else { continue }
                await self?.serialized { await $0.onEvent(pairingEvent) }
            }
        }
        // A suspended app can't hold the account store, so the session ends.
        backgroundObserver = NotificationCenter.default.addObserver(
            forName: UIApplication.didEnterBackgroundNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            Task { @MainActor in await self?.endOnBackground() }
        }
    }

    /// Callers verify device auth before showing a code.
    public func startShowingCode() async {
        await serialized { manager in
            await manager.cancelActiveSession()
            manager.state = .startingCode
            do {
                let qr = try await manager.credentialsManager.startPairing()
                manager.state = .showingCode(
                    url: qr.url,
                    sessionId: qr.sessionId,
                    expiresAt: .now + Self.codeLifetime
                )
            } catch {
                await manager.fail(with: PairingError(error))
            }
        }
    }

    public func confirmCode() async {
        await serialized { manager in
            guard case let .confirmCode(_, sessionId) = manager.state else { return }
            manager.state = .sharingAccount
            do {
                try await manager.credentialsManager.verifiedPairingPin(sessionId: sessionId)
                manager.logger.info("Account shared, session \(sessionId)")
                manager.state = .accountShared
                await manager.credentialsManager.closePairingController()
            } catch {
                await manager.fail(with: PairingError(error))
            }
        }
    }

    /// Not cancellable, since the core keeps the session.
    public func onScannedKey(_ publicKey: String) async -> Result<Void, PairingError> {
        let result = await serialized { manager -> Result<Void, PairingError> in
            await manager.cancelActiveSession()
            do {
                let confirmation = try await manager.credentialsManager.scannedPairingKey(publicKey)
                manager.state = .waitingForApproval(pin: confirmation.pinCode, sessionId: confirmation.sessionId)
                return .success(())
            } catch {
                manager.state = .idle
                await manager.credentialsManager.closePairingController()
                return .failure(PairingError(error))
            }
        }
        if Task.isCancelled, case .success = result {
            await cancel()
            return .failure(.sessionEnded)
        }
        return result
    }

    public func cancel() async {
        await serialized { manager in
            await manager.cancelActiveSession()
            manager.state = .idle
        }
    }
}

private extension PairingManager {
    /// One at a time, outside the caller's task.
    func serialized<T: Sendable>(_ operation: @escaping @MainActor (PairingManager) async -> T) async -> T {
        let previous = queue
        let task = Task { @MainActor [unowned self] in
            await previous?.value
            return await operation(self)
        }
        queue = Task { _ = await task.value }
        return await task.value
    }

    func cancelActiveSession() async {
        if state.activeSessionId != nil {
            state = .idle
            do {
                try await credentialsManager.cancelPairingProcess()
            } catch {
                logger.warning("Cancel failed: \(PairingError(error))")
            }
        }
        await credentialsManager.closePairingController()
    }

    func fail(with error: PairingError) async {
        state = .failed(error)
        await credentialsManager.closePairingController()
    }

    func endOnBackground() async {
        await serialized { manager in
            guard manager.state.activeSessionId != nil else { return }
            await manager.cancelActiveSession()
            manager.state = .failed(.sessionEnded)
        }
    }

    func onEvent(_ event: PairingEvent) async {
        // Ignores ended sessions, including the event that follows our own cancel.
        guard state.activeSessionId == event.sessionId else { return }
        logger.info("Pairing event in \(state)")
        switch event.result {
        case let .pinCode(pin):
            guard case let .showingCode(_, sessionId, _) = state else { return }
            state = .confirmCode(pin: pin, sessionId: sessionId)
        case .storedMnemonic:
            guard case .waitingForApproval = state else { return }
            // Login processing registers this device.
            await credentialsManager.closePairingController()
            state = .signedIn
        case .timedOut:
            await credentialsManager.closePairingController()
            state = .expired
        case .cancelled:
            await fail(with: .sessionEnded)
        case .error:
            await fail(with: .unknown)
        }
    }
}

extension PairingError {
    init(_ error: Error) {
        if let reason = error as? VPNErrorReason {
            if case .accountStoreBusy = reason {
                self = .vpnConnected
            } else {
                self = .unknown
            }
            return
        }
        guard let vpnError = error as? VpnError else {
            self = .unknown
            return
        }
        switch vpnError {
        case .InvalidPublicKey:
            self = .invalidCode
        case .ExistingAccount:
            self = .alreadySignedIn
        case .NoAccountStored:
            self = .notSignedIn
        case .PairingNotStarted, .PairingSessionIdMismatch, .NoPairingSharedSecret:
            self = .sessionEnded
        case .NetworkConnectionError:
            self = .offline
        case .VpnApi, .VpnApiTimeout, .UnexpectedVpnApiResponse:
            self = .server
        default:
            self = .unknown
        }
    }
}
#endif
