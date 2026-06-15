import Foundation

public enum GatewayCacheReloadPolicy {
    public static let defaultTTLSeconds: TimeInterval = 600
    public static let mainnetEnv = "mainnet"

    public static func needsReload(
        store: GatewayNodeStore,
        currentEnv: String,
        now: Date = Date(),
        ttlSeconds: TimeInterval = defaultTTLSeconds
    ) -> Bool {
        guard let fetchedForEnv = store.fetchedForEnv, fetchedForEnv == currentEnv else {
            return true
        }
        guard let lastFetchDate = store.lastFetchDate else {
            return true
        }
        return now.timeIntervalSince(lastFetchDate) > ttlSeconds
    }

    /// Bundled gateway JSON is a mainnet snapshot; never hydrate it for other envs.
    public static func shouldLoadPrebundledFallback(currentEnv: String) -> Bool {
        currentEnv == mainnetEnv
    }

    public static func persistedGatewayStoreValue(for store: GatewayNodeStore) -> String? {
        guard !store.entry.isEmpty || !store.exit.isEmpty || !store.vpn.isEmpty else {
            return nil
        }
        let raw = store.rawValue
        return raw.isEmpty ? nil : raw
    }
}
