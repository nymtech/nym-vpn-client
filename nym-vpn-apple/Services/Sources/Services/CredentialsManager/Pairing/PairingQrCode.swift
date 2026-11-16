import Foundation

/// Shape check only; spec in nym-vpn-android/docs/qr-pairing-format.md.
public enum PairingQrCode {
    static let maxLength = 512

    private static let scheme = "https"
    private static let host = "nym.com"
    private static let path = "/qr-pairing"
    private static let keyParam = "pubkey="
    private static let keyLength = 32...44
    private static let base58Alphabet = Set("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz")

    public static func parsePublicKey(_ contents: String) -> String? {
        guard contents.count <= maxLength,
              !contents.contains(where: { $0.isWhitespace || $0.isNewline }),
              let components = URLComponents(string: contents),
              components.scheme == scheme,
              components.user == nil,
              components.password == nil,
              components.host == host,
              components.port == nil,
              components.percentEncodedPath == path,
              components.fragment == nil,
              let query = components.percentEncodedQuery,
              query.hasPrefix(keyParam)
        else {
            return nil
        }
        let key = String(query.dropFirst(keyParam.count))
        guard keyLength.contains(key.count), key.allSatisfy({ base58Alphabet.contains($0) }) else {
            return nil
        }
        return key
    }
}
