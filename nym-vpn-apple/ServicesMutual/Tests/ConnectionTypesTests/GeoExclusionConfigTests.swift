import Foundation
import Testing
@testable import ConnectionTypes

@Suite struct GeoExclusionConfigTests {
    typealias Config = GeoExclusionConfig

    // MARK: - isValidPort

    @Test func validPortAcceptsInRangeExceptForbidden() {
        #expect(Config.isValidPort(1024))
        #expect(Config.isValidPort(1081))
        #expect(Config.isValidPort(8080))
        #expect(Config.isValidPort(65535))
    }

    @Test func validPortRejectsForbiddenAndOutOfRange() {
        #expect(!Config.isValidPort(1080)) // reserved SOCKS port
        #expect(!Config.isValidPort(1023)) // below min
        #expect(!Config.isValidPort(0))
    }

    // MARK: - sanitizedPort (daemon load coercion)

    @Test func sanitizedPortKeepsValidValues() {
        #expect(Config.sanitizedPort(1081) == 1081)
        #expect(Config.sanitizedPort(8080) == 8080)
        #expect(Config.sanitizedPort(65535) == 65535)
    }

    @Test func sanitizedPortCoercesInvalidToDefault() {
        #expect(Config.sanitizedPort(1080) == Config.defaultPort) // forbidden → 1081
        #expect(Config.sanitizedPort(0) == Config.defaultPort)
        #expect(Config.sanitizedPort(1023) == Config.defaultPort)
        #expect(Config.defaultPort == 1081)
    }

    // MARK: - sanitizedCountries

    @Test func sanitizedCountriesFallsBackToDefaultWhenEmpty() {
        #expect(Config.sanitizedCountries([]) == ["CN"])
    }

    @Test func sanitizedCountriesKeepsProvided() {
        #expect(Config.sanitizedCountries(["US", "GB"]) == ["US", "GB"])
    }

    // MARK: - sanitizedPortText (custom-port field, digits only)

    @Test func sanitizedPortTextStripsNonDigits() {
        #expect(Config.sanitizedPortText("12ab3") == "123")
        #expect(Config.sanitizedPortText("10 80") == "1080")
        #expect(Config.sanitizedPortText("abcd") == "")
    }

    @Test func sanitizedPortTextCapsAtFiveDigits() {
        #expect(Config.sanitizedPortText("1234567") == "12345")
        #expect(Config.sanitizedPortText("65535") == "65535")
    }

    // MARK: - validate(portText:)

    @Test func validateEmptyOrWhitespaceIsEmpty() {
        #expect(Config.validate(portText: "") == .empty)
        #expect(Config.validate(portText: "   ") == .empty)
    }

    @Test func validateForbiddenPort() {
        #expect(Config.validate(portText: "1080") == .forbidden)
    }

    @Test func validateOutOfRange() {
        #expect(Config.validate(portText: "1023") == .outOfRange)
        #expect(Config.validate(portText: "65536") == .outOfRange) // overflows UInt16
        #expect(Config.validate(portText: "abc") == .outOfRange)
    }

    @Test func validateValidPorts() {
        #expect(Config.validate(portText: "1024") == .valid)
        #expect(Config.validate(portText: "1081") == .valid)
        #expect(Config.validate(portText: "65535") == .valid)
    }

    // MARK: - defaults

    @Test func defaultInitMatchesBetaDefaults() {
        let config = Config()
        #expect(config.isEnabled == false)
        #expect(config.listenPort == 1081)
        #expect(config.excludedCountries == ["CN"])
    }

    // MARK: - Codable

    @Test func codableRoundTrips() throws {
        let original = Config(isEnabled: true, listenPort: 9050, excludedCountries: ["CN", "RU"])
        let data = try JSONEncoder().encode(original)
        let decoded = try JSONDecoder().decode(Config.self, from: data)
        #expect(decoded == original)
    }

    @Test func decodeFillsMissingKeysWithDefaults() throws {
        let data = Data("{}".utf8)
        let decoded = try JSONDecoder().decode(Config.self, from: data)
        #expect(decoded.isEnabled == false)
        #expect(decoded.listenPort == 1081)
        #expect(decoded.excludedCountries == ["CN"])
    }
}
