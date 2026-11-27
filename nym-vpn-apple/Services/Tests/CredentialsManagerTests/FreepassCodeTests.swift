import NymVPNLib
import Testing
@testable import CredentialsManager

struct FreepassCodeTests {
    @Test(arguments: [
        ("TESTCODE01", "TESTCODE01"),
        ("  abcd_1234 \n", "abcd_1234"),
        ("https://nym.com/account/freepass?code=TESTCODE01", "TESTCODE01"),
        ("https://www.nym.com/account/freepass?code=TESTCODE01", "TESTCODE01"),
        ("https://nym.com/account/freepass?utm_source=qr&code=TESTCODE01", "TESTCODE01"),
        ("nym.com/account/freepass?code=TESTCODE01", "TESTCODE01"),
        ("https://nym.com/download#code=TESTCODE01", "TESTCODE01"),
        ("nym.com/download#code=TESTCODE01", "TESTCODE01"),
        ("https://nym.com/download/#code=TESTCODE02", "TESTCODE02"),
        ("https://nym.com/account/freepass/?code=TESTCODE01", "TESTCODE01"),
        ("www.nym.com/download?utm_source=qr#code=TESTCODE01", "TESTCODE01")
    ])
    func acceptsBareCodesAndRedemptionLinks(contents: String, expected: String) {
        #expect(FreepassCode.parse(contents) == expected)
    }

    @Test(arguments: [
        "",
        "abc",
        String(repeating: "a", count: 31),
        "TEST-CODE",
        "http://nym.com/account/freepass?code=TESTCODE01",
        "https://nym.com.evil.example/account/freepass?code=TESTCODE01",
        "https://user@nym.com/account/freepass?code=TESTCODE01",
        "https://nym.com:8443/account/freepass?code=TESTCODE01",
        "https://nym.com/account/other?code=TESTCODE01",
        "https://nym.com/account/freepass?code=TESTCODE01#frag",
        "https://nym.com/account/freepass?code=TEST%2DCODE",
        "https://nym.com/account/freepass",
        "https://nym.com/download?code=TESTCODE01",
        "https://nym.com/download#code=TEST-CODE",
        "http://nym.com/download#code=TESTCODE01",
        "evil.example/download#code=TESTCODE01",
        "nym.com.evil.example/download#code=TESTCODE01",
        "https://nym.com/qr-pairing?pubkey=9Ez7Qw3hLk2sVtYpRbN4cJmA6uXfD8gH1iKoP5rStUvW"
    ])
    func rejectsAnythingElse(contents: String) {
        #expect(FreepassCode.parse(contents) == nil)
    }

#if os(iOS)
    @Test func mapsCoreErrors() {
        #expect(FreepassError(VpnError.FreepassInvalid) == .invalidCode)
        #expect(FreepassError(VpnError.FreepassAlreadyRedeemed) == .alreadyRedeemed)
        #expect(FreepassError(VpnError.RecurringSubscriptionActive) == .subscriptionActive)
        #expect(FreepassError(VpnError.RateLimited) == .rateLimited)
        #expect(FreepassError(VpnError.VpnApiTimeout) == .server)
    }
#endif

    @Test func finalErrorsAreNotRetried() {
        #expect(FreepassError.invalidCode.isFinal)
        #expect(FreepassError.alreadyRedeemed.isFinal)
        #expect(FreepassError.subscriptionActive.isFinal)
        #expect(!FreepassError.offline.isFinal)
        #expect(!FreepassError.vpnConnected.isFinal)
    }
}
