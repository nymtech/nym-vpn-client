import Testing
@testable import ConnectionTypes

struct SmokeTests {
    @Test func connectionTypesImports() {
        // Proves the test target compiles + links ConnectionTypes (and, on macOS,
        // the NymVPNLib framework produced by BuildCore.sh).
        #expect(Bool(true))
    }
}
