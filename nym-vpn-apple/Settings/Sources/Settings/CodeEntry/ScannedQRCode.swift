import CredentialsManager

public enum ScannedQRCode: Equatable, CustomStringConvertible {
    case pairing(publicKey: String)
    case freepass(code: String)

    public static func parse(_ contents: String) -> ScannedQRCode? {
        if PairingQrCode.isPairingEnabled, let publicKey = PairingQrCode.parsePublicKey(contents) {
            return .pairing(publicKey: publicKey)
        }
        if let code = FreepassCode.parse(contents) {
            return .freepass(code: code)
        }
        return nil
    }

    // Keeps codes out of logs.
    public var description: String {
        switch self {
        case .pairing:
            "pairing(<redacted>)"
        case .freepass:
            "freepass(<redacted>)"
        }
    }
}
