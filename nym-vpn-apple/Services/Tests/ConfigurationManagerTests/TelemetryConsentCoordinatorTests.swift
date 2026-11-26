import Foundation
import Testing
@testable import ConfigurationManager

@MainActor
private final class FakeDaemon: TelemetryConsentDaemon {
    var isConnected = true
    var values: [TelemetryConsentSetting: Bool] = [.networkStatistics: true, .errorReporting: true]
    /// Delay for the next write of a setting, consumed in order.
    var writeDelays: [TelemetryConsentSetting: [Duration]] = [:]
    var readDelay: Duration = .zero
    /// Number of upcoming writes of a setting that fail.
    var failingWrites: [TelemetryConsentSetting: Int] = [:]
    private(set) var writes: [(TelemetryConsentSetting, Bool)] = []

    func read(_ setting: TelemetryConsentSetting) async throws -> Bool {
        let value = values[setting] ?? false
        if readDelay > .zero { try await Task.sleep(for: readDelay) }
        return value
    }

    func write(_ setting: TelemetryConsentSetting, _ isEnabled: Bool) async throws {
        if var delays = writeDelays[setting], !delays.isEmpty {
            let delay = delays.removeFirst()
            writeDelays[setting] = delays
            try await Task.sleep(for: delay)
        }
        if let failures = failingWrites[setting], failures > 0 {
            failingWrites[setting] = failures - 1
            throw FakeError.rejected
        }
        writes.append((setting, isEnabled))
        values[setting] = isEnabled
    }
}

private enum FakeError: Error { case rejected }

@MainActor
private final class FakeStore: TelemetryConsentStore {
    var isOnboardingAnswered: Bool
    var values: [TelemetryConsentSetting: Bool] = [.networkStatistics: false, .errorReporting: false]
    var pending: [TelemetryConsentSetting: Bool]

    init(onboardingAnswered: Bool, pending: Bool) {
        isOnboardingAnswered = onboardingAnswered
        self.pending = [.networkStatistics: pending, .errorReporting: pending]
    }

    func value(_ setting: TelemetryConsentSetting) -> Bool { values[setting] ?? false }
    func setValue(_ setting: TelemetryConsentSetting, _ isEnabled: Bool) { values[setting] = isEnabled }
    func isPendingForDaemon(_ setting: TelemetryConsentSetting) -> Bool { pending[setting] ?? false }
    func setPendingForDaemon(_ setting: TelemetryConsentSetting, _ isPending: Bool) { pending[setting] = isPending }
}

@MainActor
struct TelemetryConsentCoordinatorTests {
    private func makeCoordinator(
        _ daemon: FakeDaemon,
        _ store: FakeStore,
        retryDelays: [Duration] = [.milliseconds(20)]
    ) -> TelemetryConsentCoordinator {
        TelemetryConsentCoordinator(daemon: daemon, store: store, retryDelays: retryDelays, log: { _ in })
    }

    // Pending is true before onboarding in practice; `pending: false` pins mirror()'s own check.
    @Test(arguments: [true, false])
    func beforeOnboardingConnectSendsAndMirrorsNothing(pending: Bool) async {
        let daemon = FakeDaemon()
        let store = FakeStore(onboardingAnswered: false, pending: pending)
        let coordinator = makeCoordinator(daemon, store)

        await coordinator.daemonConnected().value

        #expect(daemon.writes.isEmpty)
        #expect(store.values[.networkStatistics] == false)
    }

    @Test func onboardingAnswersAreDelivered() async {
        let daemon = FakeDaemon()
        let store = FakeStore(onboardingAnswered: false, pending: false)
        store.values[.errorReporting] = true
        let coordinator = makeCoordinator(daemon, store)

        await coordinator.onboardingAnswered().value

        #expect(daemon.values[.networkStatistics] == false)
        #expect(daemon.values[.errorReporting] == true)
        #expect(store.pending.values.allSatisfy { !$0 })
    }

    @Test func migrationSendsSavedChoiceOnceThenMirrors() async {
        let daemon = FakeDaemon()
        let store = FakeStore(onboardingAnswered: true, pending: true)
        let coordinator = makeCoordinator(daemon, store)

        await coordinator.daemonConnected().value
        #expect(daemon.values[.networkStatistics] == false)
        #expect(daemon.values[.errorReporting] == false)
        #expect(daemon.writes.count == 2)

        daemon.values[.networkStatistics] = true // changed by another client
        await coordinator.daemonConnected().value
        #expect(daemon.writes.count == 2)
        #expect(store.values[.networkStatistics] == true)
    }

    @Test func failedDeliveryAttemptsBothAndRetriesWhileConnected() async throws {
        let daemon = FakeDaemon()
        daemon.failingWrites[.networkStatistics] = 1
        let store = FakeStore(onboardingAnswered: true, pending: true)
        let coordinator = makeCoordinator(daemon, store)

        await coordinator.daemonConnected().value
        #expect(daemon.values[.errorReporting] == false)
        #expect(store.pending[.networkStatistics] == true)

        try await Task.sleep(for: .milliseconds(200))
        #expect(daemon.values[.networkStatistics] == false)
        #expect(store.pending[.networkStatistics] == false)
    }

    @Test func userWritesReachTheDaemonInOrder() async {
        let daemon = FakeDaemon()
        daemon.writeDelays[.errorReporting] = [.milliseconds(80), .zero]
        let store = FakeStore(onboardingAnswered: true, pending: false)
        let coordinator = makeCoordinator(daemon, store)

        // The first write is slow: without ordering the second would land first and the
        // first would then overwrite it.
        let first = coordinator.userChanged(.errorReporting, to: true)
        let second = coordinator.userChanged(.errorReporting, to: false)
        await first.value
        await second.value

        #expect(daemon.values[.errorReporting] == false)
        #expect(store.values[.errorReporting] == false)
    }

    @Test func aReadDoesNotOverwriteANewerUserChange() async {
        let daemon = FakeDaemon()
        daemon.readDelay = .milliseconds(80)
        let store = FakeStore(onboardingAnswered: true, pending: false)
        let coordinator = makeCoordinator(daemon, store)

        // Reads run statistics (0-80 ms) then error reporting (80-160 ms); change mid-read.
        let connect = coordinator.daemonConnected()
        try? await Task.sleep(for: .milliseconds(100))
        let change = coordinator.userChanged(.errorReporting, to: false)
        await connect.value
        await change.value

        #expect(store.values[.errorReporting] == false)
        #expect(daemon.values[.errorReporting] == false)
    }

    @Test func aChangeToOneSettingDoesNotDiscardTheOthersRead() async {
        let daemon = FakeDaemon()
        daemon.readDelay = .milliseconds(80)
        let store = FakeStore(onboardingAnswered: true, pending: false)
        let coordinator = makeCoordinator(daemon, store)

        let connect = coordinator.daemonConnected()
        try? await Task.sleep(for: .milliseconds(100)) // error-reporting read in flight
        let change = coordinator.userChanged(.networkStatistics, to: false)
        await connect.value
        await change.value

        #expect(store.values[.errorReporting] == true)
    }

    @Test func aFailedWriteShowsTheDaemonsValue() async {
        let daemon = FakeDaemon()
        daemon.failingWrites[.networkStatistics] = 1
        let store = FakeStore(onboardingAnswered: true, pending: false)
        store.values[.networkStatistics] = true
        let coordinator = makeCoordinator(daemon, store)

        await coordinator.userChanged(.networkStatistics, to: false).value

        #expect(daemon.values[.networkStatistics] == true)
        #expect(store.values[.networkStatistics] == true)
    }

    @Test func beforeOnboardingAConfigChangeSendsNothing() async {
        let daemon = FakeDaemon()
        let store = FakeStore(onboardingAnswered: false, pending: true)
        let coordinator = makeCoordinator(daemon, store)

        // The daemon emits this on any config save, e.g. a server change during onboarding.
        await coordinator.daemonConfigChanged().value

        #expect(daemon.writes.isEmpty)
    }

    @Test func aSuccessfulUserWriteEndsAPendingDelivery() async {
        let daemon = FakeDaemon()
        let store = FakeStore(onboardingAnswered: true, pending: true)
        let coordinator = makeCoordinator(daemon, store)

        await coordinator.userChanged(.networkStatistics, to: false).value
        #expect(store.pending[.networkStatistics] == false)

        daemon.values[.networkStatistics] = true // changed by another client
        await coordinator.daemonConfigChanged().value
        #expect(store.values[.networkStatistics] == true)
    }

    @Test func aChoiceThatFailsWhileDisconnectedIsSentOnReconnect() async {
        let daemon = FakeDaemon()
        daemon.isConnected = false
        daemon.failingWrites[.networkStatistics] = 1
        let store = FakeStore(onboardingAnswered: true, pending: false)
        store.values[.networkStatistics] = true
        let coordinator = makeCoordinator(daemon, store)

        await coordinator.userChanged(.networkStatistics, to: false).value
        #expect(store.pending[.networkStatistics] == true)

        daemon.isConnected = true
        await coordinator.daemonConnected().value
        #expect(daemon.values[.networkStatistics] == false)
        #expect(store.values[.networkStatistics] == false)
    }

    @Test func aReconnectRestoresTheRetryBudget() async throws {
        let daemon = FakeDaemon()
        daemon.failingWrites[.networkStatistics] = 3
        let store = FakeStore(onboardingAnswered: true, pending: true)
        let coordinator = makeCoordinator(daemon, store) // one retry per connect

        await coordinator.daemonConnected().value // fails, retries once
        try await Task.sleep(for: .milliseconds(150)) // the retry fails too; gives up
        #expect(store.pending[.networkStatistics] == true)

        await coordinator.daemonConnected().value // fails again, retries once more
        try await Task.sleep(for: .milliseconds(150))
        #expect(daemon.values[.networkStatistics] == false)
        #expect(store.pending[.networkStatistics] == false)
    }
}
