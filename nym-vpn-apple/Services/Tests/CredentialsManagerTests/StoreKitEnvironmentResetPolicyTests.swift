import Testing
import AccountPrefetchGates

struct StoreKitEnvironmentResetPolicyTests {
    @Test func syncsAppStoreOnSantaEnvironmentChange() {
        #expect(
            StoreKitEnvironmentResetPolicy.shouldSyncAppStoreOnEnvironmentChange(isSantaBuild: true)
        )
    }

    @Test func skipsAppStoreSyncOutsideSantaBuilds() {
        #expect(
            !StoreKitEnvironmentResetPolicy.shouldSyncAppStoreOnEnvironmentChange(isSantaBuild: false)
        )
    }
}
