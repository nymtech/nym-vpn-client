import Testing
@testable import ConnectionTypes

@Suite struct EnvironmentDidChangeDispatchTests {
    @Test func invokesObserversInPhaseOrderRegardlessOfRegistrationOrder() {
        var log: [EnvironmentDidChangePhase] = []

        EnvironmentDidChangeDispatch.invokeInOrder([
            (phase: .gateways, action: { log.append(.gateways) }),
            (phase: .connectionConfig, action: { log.append(.connectionConfig) }),
            (phase: .featureFlags, action: { log.append(.featureFlags) })
        ])

        #expect(log == [.connectionConfig, .featureFlags, .gateways])
    }

    @Test func invokesAllRegisteredObservers() {
        var count = 0

        EnvironmentDidChangeDispatch.invokeInOrder([
            (phase: .connectionConfig, action: { count += 1 }),
            (phase: .featureFlags, action: { count += 1 }),
            (phase: .gateways, action: { count += 1 })
        ])

        #expect(count == 3)
    }
}
