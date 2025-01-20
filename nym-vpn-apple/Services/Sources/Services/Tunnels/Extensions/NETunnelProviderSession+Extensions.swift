import NetworkExtension

extension NETunnelProviderSession {
    public func sendProviderMessageAsync(_ message: Data) async throws -> Data? {
        try await withCheckedThrowingContinuation { continuation in
            do {
                try self.sendProviderMessage(message) { response in
                    if let response {
                        continuation.resume(returning: response)
                    } else {
                        continuation.resume(returning: nil)
                    }
                }
            } catch {
                continuation.resume(throwing: error)
            }
        }
    }
}
