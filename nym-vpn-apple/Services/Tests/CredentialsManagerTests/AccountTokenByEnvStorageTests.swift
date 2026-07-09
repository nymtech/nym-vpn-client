import Foundation
import Testing
@testable import AppSettings

struct AccountTokenByEnvStorageTests {
    @Test func legacyTokenMigratesToMainnet() {
        let storage = AccountTokenByEnvStorage.load(
            encodedJSON: "{}",
            legacyToken: "740bf07e-9f8c-425b-9698-79c4a473f429"
        )
        #expect(
            storage.token(for: AccountTokenByEnvStorage.legacyMigrationEnvironment)
                == "740bf07e-9f8c-425b-9698-79c4a473f429"
        )
        #expect(storage.token(for: "sandbox") == nil)
    }

    @Test func envScopedTokensRoundTripJSON() {
        var storage = AccountTokenByEnvStorage()
        storage.setToken("740bf07e-9f8c-425b-9698-79c4a473f429", for: "mainnet")
        storage.setToken("d7dd467f-ea13-433c-a47c-3f4ca672a26b", for: "sandbox")
        let reloaded = AccountTokenByEnvStorage.load(
            encodedJSON: storage.encodedJSON(),
            legacyToken: nil
        )
        #expect(reloaded.token(for: "mainnet") == "740bf07e-9f8c-425b-9698-79c4a473f429")
        #expect(reloaded.token(for: "sandbox") == "d7dd467f-ea13-433c-a47c-3f4ca672a26b")
    }
}
