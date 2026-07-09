import Testing
@testable import ConnectionTypes

@Suite("GatewayNode operatorFamilyName")
struct GatewayNodeTests {
    @Test("carries operatorFamilyName through init")
    func carriesFamilyName() {
        let node = GatewayNode(
            id: "abc",
            location: nil,
            performance: nil,
            mixnetScore: .noScore,
            name: "node",
            description: nil,
            buildVersion: nil,
            ipv4s: [],
            ipv6s: [],
            bridges: nil,
            operatorFamilyName: "Savoy Nodes"
        )
        #expect(node.operatorFamilyName == "Savoy Nodes")
    }

    @Test("defaults operatorFamilyName to nil")
    func defaultsNil() {
        let node = GatewayNode(
            id: "abc",
            location: nil,
            performance: nil,
            mixnetScore: .noScore,
            name: nil,
            description: nil,
            buildVersion: nil,
            ipv4s: [],
            ipv6s: [],
            bridges: nil
        )
        #expect(node.operatorFamilyName == nil)
    }
}
