import Foundation

/// One of the two data-sharing choices the user makes.
public enum TelemetryConsentSetting: CaseIterable, Sendable {
    case networkStatistics
    case errorReporting
}

/// The daemon side. On macOS this is nym-vpnd, the source of truth.
@MainActor
protocol TelemetryConsentDaemon: AnyObject {
    var isConnected: Bool { get }
    func read(_ setting: TelemetryConsentSetting) async throws -> Bool
    func write(_ setting: TelemetryConsentSetting, _ isEnabled: Bool) async throws
}

/// The app side: the values the toggles show, whether onboarding has been answered, and
/// which choices the daemon hasn't received yet.
@MainActor
protocol TelemetryConsentStore: AnyObject {
    var isOnboardingAnswered: Bool { get }
    func value(_ setting: TelemetryConsentSetting) -> Bool
    func setValue(_ setting: TelemetryConsentSetting, _ isEnabled: Bool)
    func isPendingForDaemon(_ setting: TelemetryConsentSetting) -> Bool
    func setPendingForDaemon(_ setting: TelemetryConsentSetting, _ isPending: Bool)
}

/// Keeps the toggles and the daemon in agreement:
/// - every daemon read and write runs on one serial chain, in request order;
/// - the app writes only the user's own changes, plus each choice the daemon hasn't received
///   yet (onboarding answers, or the one-time migration of older installs);
/// - otherwise the toggles show the daemon's value, except for a setting with a write still
///   queued, so a read can never overwrite a newer user change.
@MainActor
final class TelemetryConsentCoordinator {
    private let daemon: TelemetryConsentDaemon
    private let store: TelemetryConsentStore
    private let retryDelays: [Duration]
    private let log: (String) -> Void

    private var chainTail: Task<Void, Never>?
    private var queuedWrites: [TelemetryConsentSetting: Int] = [:]
    private var retryAttempt = 0
    private var retryTask: Task<Void, Never>?
    /// Set by `onboardingAnswered()`, so its delivery doesn't depend on when the caller marks
    /// onboarding done in the store.
    private var answeredInThisSession = false

    /// Nothing is sent or mirrored until onboarding has asked the question.
    private var isOnboardingAnswered: Bool {
        answeredInThisSession || store.isOnboardingAnswered
    }

    init(
        daemon: TelemetryConsentDaemon,
        store: TelemetryConsentStore,
        retryDelays: [Duration] = [.seconds(5), .seconds(30), .seconds(120)],
        log: @escaping (String) -> Void
    ) {
        self.daemon = daemon
        self.store = store
        self.retryDelays = retryDelays
        self.log = log
    }

    /// The daemon connected. Until onboarding has been answered nothing is sent or mirrored
    /// (checked in `deliverPending` and `mirror`): onboarding asks the question, and
    /// `onboardingAnswered()` delivers the answers.
    @discardableResult
    func daemonConnected() -> Task<Void, Never> {
        retryAttempt = 0
        return enqueue { [weak self] in
            await self?.deliverPending()
            await self?.mirror(TelemetryConsentSetting.allCases)
        }
    }

    /// The daemon reported a config change. Only network statistics are part of that config.
    @discardableResult
    func daemonConfigChanged() -> Task<Void, Never> {
        enqueue { [weak self] in
            await self?.deliverPending()
            await self?.mirror([.networkStatistics])
        }
    }

    /// The user answered onboarding: deliver both answers now, or on the next connect.
    @discardableResult
    func onboardingAnswered() -> Task<Void, Never> {
        answeredInThisSession = true
        for setting in TelemetryConsentSetting.allCases {
            store.setPendingForDaemon(setting, true)
        }
        retryAttempt = 0
        return enqueue { [weak self] in
            await self?.deliverPending()
            await self?.mirror(TelemetryConsentSetting.allCases)
        }
    }

    /// The user flipped a toggle. Shown at once. If the write fails and the daemon's value can be
    /// read, the toggle shows it; otherwise the choice is kept and delivered later.
    @discardableResult
    func userChanged(_ setting: TelemetryConsentSetting, to isEnabled: Bool) -> Task<Void, Never> {
        store.setValue(setting, isEnabled)
        queuedWrites[setting, default: 0] += 1
        return enqueue { [weak self] in
            guard let self else { return }
            var failed = false
            do {
                try await daemon.write(setting, isEnabled)
                // The daemon now has the user's latest choice, which supersedes any undelivered one.
                store.setPendingForDaemon(setting, false)
            } catch {
                log("Failed to set \(setting) in daemon: \(error)")
                failed = true
            }
            queuedWrites[setting, default: 1] -= 1
            if failed {
                await recoverFromFailedWrite(setting)
            }
        }
    }
}

private extension TelemetryConsentCoordinator {
    @discardableResult
    func enqueue(_ operation: @escaping @MainActor () async -> Void) -> Task<Void, Never> {
        let previous = chainTail
        let task = Task { @MainActor in
            await previous?.value
            await operation()
        }
        chainTail = task
        return task
    }

    /// Sends each choice the daemon hasn't received yet, independently, and retries later if
    /// any fails.
    func deliverPending() async {
        guard isOnboardingAnswered else { return }
        let pending = TelemetryConsentSetting.allCases.filter { store.isPendingForDaemon($0) }
        guard !pending.isEmpty else { return }
        guard daemon.isConnected else {
            log("Daemon not connected; data-sharing choices are sent when it connects")
            return
        }
        var failed = false
        for setting in pending {
            do {
                try await daemon.write(setting, store.value(setting))
                store.setPendingForDaemon(setting, false)
            } catch {
                log("Failed to deliver \(setting) to daemon: \(error)")
                failed = true
            }
        }
        if failed {
            scheduleRetry()
        } else {
            retryAttempt = 0
        }
    }

    /// After a failed user write: show the daemon's value if it can be read, otherwise keep the
    /// user's choice for delivery. A daemon drop can fail both the write and the read while
    /// `isConnected` is still true, so a failed read must not drop the choice.
    func recoverFromFailedWrite(_ setting: TelemetryConsentSetting) async {
        if daemon.isConnected, !store.isPendingForDaemon(setting) {
            do {
                let isEnabled = try await daemon.read(setting)
                // A write queued meanwhile is newer than this value.
                if canMirror(setting), isEnabled != store.value(setting) {
                    store.setValue(setting, isEnabled)
                }
                return
            } catch {
                log("Failed to read \(setting) from daemon: \(error)")
            }
        }
        store.setPendingForDaemon(setting, true)
        if daemon.isConnected {
            scheduleRetry()
        }
    }

    /// At most one retry waits at a time, and it delivers every pending choice; further failures
    /// meanwhile neither reschedule it nor use up the budget.
    func scheduleRetry() {
        guard retryTask == nil else { return }
        guard retryAttempt < retryDelays.count else {
            log("Giving up on delivering data-sharing choices until the daemon reconnects")
            return
        }
        let delay = retryDelays[retryAttempt]
        retryAttempt += 1
        retryTask = Task { @MainActor [weak self] in
            try? await Task.sleep(for: delay)
            guard let self else { return }
            retryTask = nil
            enqueue { [weak self] in
                await self?.deliverPending()
                await self?.mirror(TelemetryConsentSetting.allCases)
            }
        }
    }

    /// Shows the daemon's value for each setting that has no undelivered choice and no queued
    /// write, checked again after the read returns.
    func mirror(_ settings: [TelemetryConsentSetting]) async {
        guard isOnboardingAnswered, daemon.isConnected else { return }
        for setting in settings where canMirror(setting) {
            do {
                let isEnabled = try await daemon.read(setting)
                guard canMirror(setting), isEnabled != store.value(setting) else { continue }
                store.setValue(setting, isEnabled)
            } catch {
                log("Failed to read \(setting) from daemon: \(error)")
            }
        }
    }

    func canMirror(_ setting: TelemetryConsentSetting) -> Bool {
        !store.isPendingForDaemon(setting) && queuedWrites[setting, default: 0] == 0
    }
}
