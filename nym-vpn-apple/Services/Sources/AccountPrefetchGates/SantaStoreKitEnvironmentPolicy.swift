import Foundation

/// Santa QA: StoreKit account expectations and env-change refresh behaviour.
public enum SantaStoreKitEnvironmentPolicy: Equatable, Sendable {
#if SANTA
    public static let isSantaBuild = true
#else
    public static let isSantaBuild = false
#endif

    public static func guidanceMessage(isTestFlight: Bool) -> String {
        if isTestFlight {
            return "TestFlight IAP uses your TestFlight Apple ID, not Developer Settings sandbox account."
        }
        return """
        Xcode debug IAP uses Settings - Developer - Sandbox Apple Account. \
        Switching VPN env here does not change the StoreKit Apple ID.
        """
    }

    public static func shouldSyncAppStoreBeforeReload() -> Bool {
        isSantaBuild
    }
}
