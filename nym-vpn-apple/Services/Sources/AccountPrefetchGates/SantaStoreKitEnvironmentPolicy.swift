import Foundation

/// Santa QA: StoreKit account expectations for the Santa menu.
public enum SantaStoreKitEnvironmentPolicy: Equatable, Sendable {
    public static func guidanceMessage(isTestFlight: Bool) -> String {
        if isTestFlight {
            return "TestFlight IAP uses your TestFlight Apple ID, not Developer Settings sandbox account."
        }
        return """
        Xcode debug IAP uses Settings - Developer - Sandbox Apple Account. \
        Switching VPN env here does not change the StoreKit Apple ID.
        """
    }
}
