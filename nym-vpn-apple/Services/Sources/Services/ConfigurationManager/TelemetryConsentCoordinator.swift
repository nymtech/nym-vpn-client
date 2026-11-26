#if os(macOS)
import Foundation

public enum TelemetryConsentSetting: CaseIterable, Sendable {
    case networkStatistics
    case errorReporting
}

@MainActor
protocol TelemetryConsentDaemon: AnyObject {
    var isConnected: Bool { get }
    func read(_ setting: TelemetryConsentSetting) async throws -> Bool
    func write(_ setting: TelemetryConsentSetting, _ isEnabled: Bool) async throws
}

@MainActor
protocol TelemetryConsentStore: AnyObject {
    var isOnboardingAnswered: Bool { get }
    func value(_ setting: TelemetryConsentSetting) -> Bool
    func setValue(_ setting: TelemetryConsentSetting, _ isEnabled: Bool)
    func isPendingForDaemon(_ setting: TelemetryConsentSetting) -> Bool
    func setPendingForDaemon(_ setting: TelemetryConsentSetting, _ isPending: Bool)
}

/// Keeps the data-sharing toggles in agreement with nym-vpnd, which owns them.
/// - Every daemon read and write runs on one serial chain, in request order.
/// - The app writes only user changes and choices the daemon hasn't received yet (onboarding
///   answers, or the one-time migration of older installs).
/// - Otherwise the toggles show the daemon's value, except while a setting has a write queued
///   or a choice undelivered, so a read never overwrites a newer user change.
/// - Nothing is sent or read before onboarding has been answered.
@MainActor
final class TelemetryConsentCoordinator {
    private let daemon: TelemetryConsentDaemon
    private let store: TelemetryConsentStore
    private let retryDelays: [Duration]
    private let sleep: (Duration) async -> Void
    private let log: (String) -> Void

    private var chainTail: Task<Void, Never>?
    private var queuedWrites: [TelemetryConsentSetting: Int] = [:]
    private var retryAttempt = 0
    private var retryTask: Task<Void, Never>?

    init(
        daemon: TelemetryConsentDaemon,
        store: TelemetryConsentStore,
        retryDelays: [Duration] = [.seconds(5), .seconds(30), .seconds(120)],
        sleep: @escaping (Duration) async -> Void = { try? await Task.sleep(for: $0) },
        log: @escaping (String) -> Void
    ) {
        self.daemon = daemon
        self.store = store
        self.retryDelays = retryDelays
        self.sleep = sleep
        self.log = log
    }

    /// The retry waiting to run, if any.
    var scheduledRetry: Task<Void, Never>? { retryTask }

    @discardableResult
    func daemonConnected() -> Task<Void, Never> {
        retryAttempt = 0
        return sync(mirroring: TelemetryConsentSetting.allCases)
    }

    /// Only network statistics are part of the daemon config this event reports.
    @discardableResult
    func daemonConfigChanged() -> Task<Void, Never> {
        sync(mirroring: [.networkStatistics])
    }

    /// Call after onboarding has been marked answered in the store.
    @discardableResult
    func onboardingAnswered() -> Task<Void, Never> {
        for setting in TelemetryConsentSetting.allCases {
            store.setPendingForDaemon(setting, true)
        }
        retryAttempt = 0
        return sync(mirroring: TelemetryConsentSetting.allCases)
    }

    @discardableResult
    func userChanged(_ setting: TelemetryConsentSetting, to isEnabled: Bool) -> Task<Void, Never> {
        store.setValue(setting, isEnabled)
        queuedWrites[setting, default: 0] += 1
        return enqueue { [self] in
            var failed = false
            do {
                try await daemon.write(setting, isEnabled)
                store.setPendingForDaemon(setting, false)
            } catch {
                log("Failed to set \(setting) in daemon: \(error)")
                failed = true
            }
            queuedWrites[setting]? -= 1
            if failed {
                await recoverFromFailedWrite(setting)
            }
        }
    }
}

private extension TelemetryConsentCoordinator {
    func enqueue(_ operation: @escaping @MainActor () async -> Void) -> Task<Void, Never> {
        let previous = chainTail
        let task = Task { @MainActor in
            await previous?.value
            await operation()
        }
        chainTail = task
        return task
    }

    func sync(mirroring settings: [TelemetryConsentSetting]) -> Task<Void, Never> {
        enqueue { [self] in
            await deliverPending()
            await mirror(settings)
        }
    }

    func deliverPending() async {
        guard store.isOnboardingAnswered else { return }
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

    /// Shows the daemon's value if it can be read; otherwise keeps the user's choice for
    /// delivery. A daemon drop can fail the write and the read before `isConnected` turns false.
    func recoverFromFailedWrite(_ setting: TelemetryConsentSetting) async {
        if daemon.isConnected, !store.isPendingForDaemon(setting) {
            do {
                let isEnabled = try await daemon.read(setting)
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

    /// One retry waits at a time and delivers everything pending, so further failures while it
    /// waits don't use up the budget.
    func scheduleRetry() {
        guard retryTask == nil else { return }
        guard retryAttempt < retryDelays.count else {
            log("Giving up on delivering data-sharing choices until the daemon reconnects")
            return
        }
        let delay = retryDelays[retryAttempt]
        retryAttempt += 1
        retryTask = Task { [weak self] in
            await self?.sleep(delay)
            guard let self else { return }
            retryTask = nil
            await sync(mirroring: TelemetryConsentSetting.allCases).value
        }
    }

    func mirror(_ settings: [TelemetryConsentSetting]) async {
        guard store.isOnboardingAnswered, daemon.isConnected else { return }
        for setting in settings where canMirror(setting) {
            do {
                let isEnabled = try await daemon.read(setting)
                // Checked again: a change may have been queued during the read.
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
#endif
