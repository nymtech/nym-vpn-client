#if SANTA
import Foundation
import Testing
@testable import ConnectionTypes

@Suite struct GatewayCacheReloadPolicyTests {
    private let now = Date(timeIntervalSince1970: 1_700_000_000)
    private let freshDate = Date(timeIntervalSince1970: 1_700_000_000)
    private let staleDate = Date(timeIntervalSince1970: 1_699_000_000)

    @Test func freshCacheSameEnvDoesNotReload() {
        let store = GatewayNodeStore(
            lastFetchDate: freshDate,
            fetchedForEnv: "mainnet"
        )
        #expect(
            GatewayCacheReloadPolicy.needsReload(
                store: store,
                currentEnv: "mainnet",
                now: now
            ) == false
        )
    }

    @Test func freshCacheEnvMismatchReloads() {
        let store = GatewayNodeStore(
            lastFetchDate: freshDate,
            fetchedForEnv: "mainnet"
        )
        #expect(
            GatewayCacheReloadPolicy.needsReload(
                store: store,
                currentEnv: "sandbox",
                now: now
            ) == true
        )
    }

    @Test func missingLastFetchDateReloads() {
        let store = GatewayNodeStore(
            lastFetchDate: nil,
            fetchedForEnv: "mainnet"
        )
        #expect(
            GatewayCacheReloadPolicy.needsReload(
                store: store,
                currentEnv: "mainnet",
                now: now
            ) == true
        )
    }

    @Test func missingFetchedForEnvReloadsEvenWhenFresh() {
        let store = GatewayNodeStore(
            lastFetchDate: freshDate,
            fetchedForEnv: nil
        )
        #expect(
            GatewayCacheReloadPolicy.needsReload(
                store: store,
                currentEnv: "mainnet",
                now: now
            ) == true
        )
    }

    @Test func expiredTTLSameEnvReloads() {
        let store = GatewayNodeStore(
            lastFetchDate: staleDate,
            fetchedForEnv: "mainnet"
        )
        #expect(
            GatewayCacheReloadPolicy.needsReload(
                store: store,
                currentEnv: "mainnet",
                now: now
            ) == true
        )
    }

    @Test func prebundledFallbackOnlyOnMainnet() {
        #expect(GatewayCacheReloadPolicy.shouldLoadPrebundledFallback(currentEnv: "mainnet") == true)
        #expect(GatewayCacheReloadPolicy.shouldLoadPrebundledFallback(currentEnv: "sandbox") == false)
        #expect(GatewayCacheReloadPolicy.shouldLoadPrebundledFallback(currentEnv: "canary") == false)
    }

    @Test func emptyStorePersistsNil() {
        let store = GatewayNodeStore()
        #expect(GatewayCacheReloadPolicy.persistedGatewayStoreValue(for: store) == nil)
    }

    @Test func clearedStoreMatchesEnvironmentChangePersist() {
        let store = GatewayNodeStore(
            lastFetchDate: freshDate,
            fetchedForEnv: "mainnet",
            entry: [
                GatewayNode(
                    id: "entry-id",
                    location: nil,
                    performance: nil,
                    mixnetScore: .noScore,
                    name: nil,
                    description: nil,
                    buildVersion: nil,
                    ipv4s: [],
                    ipv6s: [],
                    bridges: nil
                )
            ]
        )
        #expect(GatewayCacheReloadPolicy.persistedGatewayStoreValue(for: store) != nil)
        let cleared = GatewayNodeStore()
        #expect(GatewayCacheReloadPolicy.persistedGatewayStoreValue(for: cleared) == nil)
    }

    @Test func fetchedForEnvSurvivesJSONRoundTrip() throws {
        var store = GatewayNodeStore(
            lastFetchDate: freshDate,
            fetchedForEnv: "sandbox"
        )
        store.entry = [
            GatewayNode(
                id: "entry-id",
                location: nil,
                performance: nil,
                mixnetScore: .noScore,
                name: nil,
                description: nil,
                buildVersion: nil,
                ipv4s: [],
                ipv6s: [],
                bridges: nil
            )
        ]
        store.exit = [
            GatewayNode(
                id: "exit-id",
                location: nil,
                performance: nil,
                mixnetScore: .noScore,
                name: nil,
                description: nil,
                buildVersion: nil,
                ipv4s: [],
                ipv6s: [],
                bridges: nil
            )
        ]
        store.vpn = [
            GatewayNode(
                id: "vpn-id",
                location: nil,
                performance: nil,
                mixnetScore: .noScore,
                name: nil,
                description: nil,
                buildVersion: nil,
                ipv4s: [],
                ipv6s: [],
                bridges: nil
            )
        ]

        let roundTripped = try #require(GatewayNodeStore(rawValue: store.rawValue))
        #expect(roundTripped.fetchedForEnv == "sandbox")
        #expect(roundTripped.entry.count == 1)
        #expect(roundTripped.exit.count == 1)
        #expect(roundTripped.vpn.count == 1)
    }
}
#endif
