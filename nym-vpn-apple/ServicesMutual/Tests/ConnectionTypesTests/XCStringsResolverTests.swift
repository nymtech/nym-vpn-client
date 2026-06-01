import Testing

struct XCStringsResolverTests {
    @Test func resolvesKnownKeys() throws {
        let resolver = try XCStringsResolver.default()
        #expect(resolver.string("planExpiresOn") == "Plan expires on")
        #expect(resolver.string("planValidUntil") == "Plan valid until")
        #expect(resolver.string("noActivePlan") == "This account has no active subscription")
        #expect(resolver.string("settings.logout") == "Logout")
    }

    @Test func unknownKeyFallsBackToKey() throws {
        let resolver = try XCStringsResolver.default()
        #expect(resolver.string("totally.bogus.key") == "totally.bogus.key")
    }
}
