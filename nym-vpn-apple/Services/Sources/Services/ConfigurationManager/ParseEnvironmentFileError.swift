import Foundation

public struct ParseEnvironmentFileError: LocalizedError {
    public enum Kind {
        case invalidValue
        case fileNotFound
    }

    public let source: String
    public let kind: Kind

    init(kind: Kind, source: String) {
        self.kind = kind
        self.source = source
    }

    func errorDescription() -> String? {
        switch kind {
        case .invalidValue:
            return "Invalid value"
        case .fileNotFound:
            return "Env file not found"
        }
    }
}
