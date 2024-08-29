import Foundation
import Constants

public enum ConfigurationManager {
    public static func setEnvVariables(for environment: Env) throws {
        do {
            let envString = try ConfigurationManager.contentOfEnvFile(named: environment.rawValue)
            try setEnvironmentVariables(envString: envString)
        } catch {
            print(error)
        }
    }
}

private extension ConfigurationManager {
    static func contentOfEnvFile(named: String) throws -> String {
        guard let filePath = Bundle.main.path(forResource: named, ofType: "env")
        else {
            throw GeneralNymError.noEnvFile
        }
        do {
            let fileContent = try String(contentsOfFile: filePath, encoding: .utf8)
            print(fileContent)
            return fileContent
        } catch {
            throw error
        }
    }

    static func setEnvironmentVariables(envString: String) throws {
        let escapeQuote = "\""
        let lines = envString.split(whereSeparator: { $0.isNewline })

        try lines.forEach { line in
            guard !line.isEmpty else { return }

            let substrings = line.split(separator: "=", maxSplits: 2)
            if substrings.count == 2 {
                let key = substrings[0].trimmingCharacters(in: .whitespaces)
                var value = substrings[1].trimmingCharacters(in: .whitespaces)

                if value.hasPrefix(escapeQuote) && value.hasSuffix(escapeQuote) {
                    value.removeFirst()
                    value.removeLast()
                }

                setenv(key, value, 1)
            } else {
                throw ParseEnvironmentFileError(kind: .invalidValue, source: String(line))
            }
        }
    }
}
