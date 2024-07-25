import Foundation
import Constants

public class ConfigurationManager {
    public static func setEnvVariables() throws {
        let envString = try ConfigurationManager.contentOfEnvFile(named: Constants.currentEnvironment.rawValue)
        try setEnvironmentVariables(envString: envString)
    }
}

private extension ConfigurationManager {
    static func contentOfEnvFile(named: String) throws -> String {
        guard let filePath = Bundle.main.path(forResource: named, ofType: "env") else {
            throw GeneralNymError.noEnvFile
        }

        return try String(contentsOfFile: filePath, encoding: .utf8)
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
