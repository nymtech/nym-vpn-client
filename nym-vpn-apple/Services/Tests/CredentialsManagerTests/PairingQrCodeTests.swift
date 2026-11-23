import Foundation
import Testing
@testable import CredentialsManager

struct PairingQrCodeTests {
    private struct Vectors: Decodable {
        let vectors: [Vector]
    }

    private struct Vector: Decodable {
        let name: String
        let contents: String
        let publicKey: String?
    }

    private static func vectorsURL(file: StaticString = #filePath) -> URL {
        var url = URL(fileURLWithPath: "\(file)")
        for _ in 0..<5 { url.deleteLastPathComponent() }
        return url.appendingPathComponent("nym-vpn-android/app/src/test/resources/pairing/qr-pairing-vectors.json")
    }

    @Test func matchesSharedVectors() throws {
        let data = try Data(contentsOf: Self.vectorsURL())
        let vectors = try JSONDecoder().decode(Vectors.self, from: data).vectors
        #expect(!vectors.isEmpty)
        for vector in vectors {
            #expect(PairingQrCode.parsePublicKey(vector.contents) == vector.publicKey, "\(vector.name)")
        }
    }

    @Test func rejectsOversizedText() {
        let contents = "https://nym.com/qr-pairing?pubkey=" + String(repeating: "1", count: PairingQrCode.maxLength)
        #expect(PairingQrCode.parsePublicKey(contents) == nil)
    }

    @Test func stateDescriptionHidesUrlAndPin() {
        let showing = PairingState.showingCode(
            url: "https://nym.com/qr-pairing?pubkey=11111111111111111111111111111111",
            sessionId: 7,
            expiresAt: .now
        )
        let waiting = PairingState.waitingForApproval(pin: "123456", sessionId: 7)
        #expect(!showing.description.contains("pubkey"))
        #expect(!waiting.description.contains("123456"))
    }
}
