import Foundation

/// Shape check only, matching `isValidRedemptionCode` in websites www/vpn-api.
public enum FreepassCode {
    private static let scheme = "https"
    private static let hosts: Set<String> = ["nym.com", "www.nym.com"]
    private static let freepassPath = "/account/freepass"
    private static let downloadPath = "/download"
    private static let codeParam = "code="
    private static let maxLength = 512
    private static let codeLength = 4...30
    private static let codeCharacters = Set("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_")

    public static func parse(_ contents: String) -> String? {
        let trimmed = contents.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.count <= maxLength else { return nil }
        if isCode(trimmed) {
            return trimmed
        }
        // Typed links often lack the scheme.
        let link = trimmed.contains("://") ? trimmed : "\(scheme)://\(trimmed)"
        guard let components = URLComponents(string: link),
              components.scheme == scheme,
              components.user == nil,
              components.password == nil,
              let host = components.host,
              hosts.contains(host),
              components.port == nil
        else {
            return nil
        }
        let params: String?
        switch components.percentEncodedPath.removingSuffix("/") {
        case freepassPath:
            params = components.percentEncodedFragment == nil ? components.percentEncodedQuery : nil
        case downloadPath:
            params = components.percentEncodedFragment
        default:
            params = nil
        }
        guard let code = params?
            .split(separator: "&")
            .first(where: { $0.hasPrefix(codeParam) })?
            .dropFirst(codeParam.count)
        else {
            return nil
        }
        return isCode(String(code)) ? String(code) : nil
    }

    private static func isCode(_ text: String) -> Bool {
        codeLength.contains(text.count) && text.allSatisfy { codeCharacters.contains($0) }
    }
}

private extension String {
    func removingSuffix(_ suffix: String) -> String {
        hasSuffix(suffix) ? String(dropLast(suffix.count)) : self
    }
}
