import Foundation

/// StoreKit refresh behaviour when the VPN environment changes (Santa menu).
public enum StoreKitEnvironmentResetPolicy: Equatable, Sendable {
    public static func shouldSyncAppStoreOnEnvironmentChange(isSantaBuild: Bool) -> Bool {
        isSantaBuild
    }
}
