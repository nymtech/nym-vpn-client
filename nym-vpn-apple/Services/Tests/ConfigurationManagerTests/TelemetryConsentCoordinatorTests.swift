#if os(macOS)
import Foundation
import Testing
@testable import ConfigurationManager

/// Holds whatever calls `pass()` until the test calls `release()`.
@MainActor
private final class Gate {
    private var held: [CheckedContinuation<Void, Never>] = []
    private var watchers: [CheckedContinuation<Void, Never>] = []

    func pass() async {
        await withCheckedContinuation { continuation in
            held.append(continuation)
            watchers.forEach { $0.resume() }
            watchers = []
        }
    }

    /// Returns once something is held.
    func reached() async {
        guard held.isEmpty else { return }
        await withCheckedContinuation { watchers.append($0) }
    }

    func release() {
        let released = held
        held = []
        released.forEach { $0.resume() }
    }
}

private enum FakeError: Error { case rejected }

@MainActor
private final class FakeDaemon: TelemetryConsentDaemon {
    var isConnected = true
    var values: [TelemetryConsentSetting: Bool] = [.networkStatistics: true, .errorReporting: true]
    var failingWrites: [TelemetryConsentSetting: Int] = [:]
    var failingReads: [TelemetryConsentSetting: Int] = [:]
    /// Each holds the next read or write of its setting.
    var readGates: [TelemetryConsentSetting: Gate] = [:]
    var writeGates: [TelemetryConsentSetting: Gate] = [:]
    private(set) var writeCount = 0

    func read(_ setting: TelemetryConsentSetting) async throws -> Bool {
        if consume(&failingReads, setting) { throw FakeError.rejected }
        let value = values[setting] ?? false
        await readGates.removeValue(forKey: setting)?.pass()
        return value
    }

    func write(_ setting: TelemetryConsentSetting, _ isEnabled: Bool) async throws {
        await writeGates.removeValue(forKey: setting)?.pass()
        if consume(&failingWrites, setting) { throw FakeError.rejected }
        writeCount += 1
        values[setting] = isEnabled
    }

    private func consume(_ counts: inout [TelemetryConsentSetting: Int], _ setting: TelemetryConsentSetting) -> Bool {
        guard let count = counts[setting], count > 0 else { return false }
        counts[setting] = count - 1
        return true
    }
}

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

@Suite
@MainActor
struct TelemetryConsentCoordinatorTests {
    enum ReadRace: CaseIterable { case onConnect, afterFailedWrite }
    enum DaemonDrop: CaseIterable { case beforeTheWrite, duringTheWrite }

    /// Retries run without delay unless `retryGate` holds them.
    private func makeCoordinator(
        _ daemon: FakeDaemon,
        _ store: FakeStore,
        retryDelays: [Duration] = [.zero],
        retryGate: Gate? = nil
    ) -> TelemetryConsentCoordinator {
        TelemetryConsentCoordinator(
            daemon: daemon,
            store: store,
            retryDelays: retryDelays,
            sleep: { _ in await retryGate?.pass() },
            log: { _ in }
        )
    }

    private func runScheduledRetry(_ coordinator: TelemetryConsentCoordinator, _ gate: Gate) async throws {
        let retry = try #require(coordinator.scheduledRetry)
        await gate.reached()
        gate.release()
        await retry.value
    }

    @Test(arguments: [true, false])
    func nothingIsSentOrMirroredBeforeOnboarding(pending: Bool) async {
        let daemon = FakeDaemon()
        let store = FakeStore(onboardingAnswered: false, pending: pending)
        let coordinator = makeCoordinator(daemon, store)

        await coordinator.daemonConnected().value
        await coordinator.daemonConfigChanged().value

        #expect(daemon.writeCount == 0)
        #expect(store.values[.networkStatistics] == false)
    }

    @Test func onboardingAnswersAreDelivered() async {
        let daemon = FakeDaemon()
        let store = FakeStore(onboardingAnswered: true, pending: false)
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
        #expect(daemon.values == [.networkStatistics: false, .errorReporting: false])

        daemon.values[.networkStatistics] = true // changed by another client
        await coordinator.daemonConnected().value
        #expect(daemon.writeCount == 2)
        #expect(store.values[.networkStatistics] == true)
    }

    @Test func aFailedDeliveryIsRetriedWithoutBlockingTheOtherSetting() async throws {
        let daemon = FakeDaemon()
        daemon.failingWrites[.networkStatistics] = 1
        let store = FakeStore(onboardingAnswered: true, pending: true)
        let retryGate = Gate()
        let coordinator = makeCoordinator(daemon, store, retryGate: retryGate)

        await coordinator.daemonConnected().value
        #expect(daemon.values[.errorReporting] == false)
        #expect(store.pending[.networkStatistics] == true)

        try await runScheduledRetry(coordinator, retryGate)
        #expect(daemon.values[.networkStatistics] == false)
        #expect(store.pending[.networkStatistics] == false)
    }

    @Test func failuresWhileARetryWaitsDoNotUseUpTheBudget() async throws {
        let daemon = FakeDaemon()
        daemon.failingWrites[.networkStatistics] = 3
        let store = FakeStore(onboardingAnswered: true, pending: true)
        let retryGate = Gate()
        let coordinator = makeCoordinator(daemon, store, retryDelays: [.zero, .zero], retryGate: retryGate)

        await coordinator.daemonConnected().value
        await coordinator.daemonConfigChanged().value
        try await runScheduledRetry(coordinator, retryGate)
        try await runScheduledRetry(coordinator, retryGate)

        #expect(daemon.values[.networkStatistics] == false)
    }

    @Test func aReconnectRestoresTheRetryBudget() async throws {
        let daemon = FakeDaemon()
        daemon.failingWrites[.networkStatistics] = 3
        let store = FakeStore(onboardingAnswered: true, pending: true)
        let retryGate = Gate()
        let coordinator = makeCoordinator(daemon, store, retryGate: retryGate)

        await coordinator.daemonConnected().value
        try await runScheduledRetry(coordinator, retryGate)
        #expect(coordinator.scheduledRetry == nil)

        await coordinator.daemonConnected().value
        try await runScheduledRetry(coordinator, retryGate)
        #expect(daemon.values[.networkStatistics] == false)
    }

    @Test func userWritesReachTheDaemonInOrder() async {
        let daemon = FakeDaemon()
        let firstWrite = Gate()
        daemon.writeGates[.errorReporting] = firstWrite
        let store = FakeStore(onboardingAnswered: true, pending: false)
        let coordinator = makeCoordinator(daemon, store)

        let first = coordinator.userChanged(.errorReporting, to: true)
        await firstWrite.reached()
        let second = coordinator.userChanged(.errorReporting, to: false)
        firstWrite.release()
        await first.value
        await second.value

        #expect(daemon.values[.errorReporting] == false)
        #expect(store.values[.errorReporting] == false)
    }

    @Test(arguments: ReadRace.allCases)
    func aReadDoesNotOverwriteANewerChange(_ race: ReadRace) async {
        let daemon = FakeDaemon()
        let read = Gate()
        daemon.readGates[.networkStatistics] = read
        let store = FakeStore(onboardingAnswered: true, pending: false)
        store.values[.networkStatistics] = true
        let coordinator = makeCoordinator(daemon, store)

        let reading: Task<Void, Never>
        switch race {
        case .onConnect:
            reading = coordinator.daemonConnected()
        case .afterFailedWrite:
            daemon.failingWrites[.networkStatistics] = 1
            reading = coordinator.userChanged(.networkStatistics, to: false)
        }
        await read.reached()
        let newer = coordinator.userChanged(.networkStatistics, to: false)
        read.release()
        await reading.value
        await newer.value

        #expect(daemon.values[.networkStatistics] == false)
        #expect(store.values[.networkStatistics] == false)
    }

    @Test func aChangeToOneSettingDoesNotDiscardTheOthersRead() async {
        let daemon = FakeDaemon()
        let read = Gate()
        daemon.readGates[.errorReporting] = read
        let store = FakeStore(onboardingAnswered: true, pending: false)
        let coordinator = makeCoordinator(daemon, store)

        let connect = coordinator.daemonConnected()
        await read.reached()
        let change = coordinator.userChanged(.networkStatistics, to: false)
        read.release()
        await connect.value
        await change.value

        #expect(store.values[.errorReporting] == true)
    }

    @Test func failedWritesEndOnTheDaemonsValue() async {
        let daemon = FakeDaemon()
        daemon.failingWrites[.errorReporting] = 2
        let store = FakeStore(onboardingAnswered: true, pending: false)
        store.values[.errorReporting] = true
        let coordinator = makeCoordinator(daemon, store)

        let first = coordinator.userChanged(.errorReporting, to: false)
        let second = coordinator.userChanged(.errorReporting, to: false)
        await first.value
        await second.value

        #expect(daemon.values[.errorReporting] == true)
        #expect(store.values[.errorReporting] == true)
        #expect(store.pending[.errorReporting] == false)
    }

    @Test(arguments: DaemonDrop.allCases, TelemetryConsentSetting.allCases)
    func aChoiceSurvivesADaemonDrop(_ drop: DaemonDrop, _ setting: TelemetryConsentSetting) async {
        let daemon = FakeDaemon()
        daemon.failingWrites[setting] = 1
        switch drop {
        case .beforeTheWrite:
            daemon.isConnected = false
        case .duringTheWrite:
            daemon.failingReads[setting] = 1 // still reported as connected
        }
        let store = FakeStore(onboardingAnswered: true, pending: false)
        store.values[setting] = true
        let retryGate = Gate()
        let coordinator = makeCoordinator(daemon, store, retryGate: retryGate)

        await coordinator.userChanged(setting, to: false).value
        #expect(store.pending[setting] == true)

        daemon.isConnected = true
        await coordinator.daemonConnected().value
        #expect(daemon.values[setting] == false)
        #expect(store.values[setting] == false)
        retryGate.release()
    }

    @Test func aFailedWriteOfAnUndeliveredChoiceIsRetried() async throws {
        let daemon = FakeDaemon()
        daemon.failingWrites[.networkStatistics] = 1
        let store = FakeStore(onboardingAnswered: true, pending: true)
        let retryGate = Gate()
        let coordinator = makeCoordinator(daemon, store, retryGate: retryGate)

        await coordinator.userChanged(.networkStatistics, to: false).value
        try await runScheduledRetry(coordinator, retryGate)

        #expect(daemon.values[.networkStatistics] == false)
        #expect(store.pending[.networkStatistics] == false)
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
}
#endif
