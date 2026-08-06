import NetworkExtension

extension NETunnelProviderSession {
    public func sendProviderMessageAsync(_ message: Data) async throws -> Data? {
        try await withCheckedThrowingContinuation { continuation in
            do {
                try self.sendProviderMessage(message) { response in
                    continuation.resume(returning: response)
                }
            } catch {
                continuation.resume(throwing: error)
            }
        }
    }
}
