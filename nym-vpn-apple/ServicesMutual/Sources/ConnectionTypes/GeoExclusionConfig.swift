#if os(macOS)
import NymVPNLib
#endif

public struct GeoExclusionConfig: Codable, Equatable {
    public static let minPort: UInt16 = 1024
    public static let maxPort: UInt16 = 65535
    public static let forbiddenPort: UInt16 = 1080
    public static let defaultPort: UInt16 = 1081
    public static let defaultExcludedCountries = ["CN"]

    public var isEnabled: Bool
    public var listenPort: UInt16
    public var excludedCountries: [String]

    public init(
        isEnabled: Bool = false,
        listenPort: UInt16 = GeoExclusionConfig.defaultPort,
        excludedCountries: [String] = GeoExclusionConfig.defaultExcludedCountries
    ) {
        self.isEnabled = isEnabled
        self.listenPort = listenPort
        self.excludedCountries = excludedCountries
    }

#if os(macOS)
    public init(from settings: GeoExclusionSettings) {
        self.isEnabled = settings.enabled
        self.listenPort = GeoExclusionConfig.sanitizedPort(settings.listenPort)
        self.excludedCountries = GeoExclusionConfig.sanitizedCountries(settings.excludedCountries)
    }
#endif

    enum CodingKeys: String, CodingKey {
        case isEnabled
        case listenPort
        case excludedCountries
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        isEnabled = try container.decodeIfPresent(Bool.self, forKey: .isEnabled) ?? false
        listenPort = try container.decodeIfPresent(UInt16.self, forKey: .listenPort) ?? GeoExclusionConfig.defaultPort
        excludedCountries = try container.decodeIfPresent([String].self, forKey: .excludedCountries)
            ?? GeoExclusionConfig.defaultExcludedCountries
    }

    /// Validates a candidate listen port: in range and not the reserved SOCKS port.
    public static func isValidPort(_ port: UInt16) -> Bool {
        port >= minPort && port <= maxPort && port != forbiddenPort
    }

    /// Coerces a raw port (e.g. from the daemon) to a valid one, falling back to `defaultPort`.
    public static func sanitizedPort(_ port: UInt16) -> UInt16 {
        isValidPort(port) ? port : defaultPort
    }

    /// Coerces a raw country list, falling back to the beta default when empty.
    public static func sanitizedCountries(_ countries: [String]) -> [String] {
        countries.isEmpty ? defaultExcludedCountries : countries
    }

    /// Keeps only digits, capped at 5 (max port is 5 digits). For the custom-port text field.
    public static func sanitizedPortText(_ text: String) -> String {
        String(text.filter(\.isNumber).prefix(5))
    }

    public enum PortValidation: Equatable {
        case valid
        case empty
        case outOfRange
        case forbidden
    }

    /// Validates raw text from the custom-port field.
    public static func validate(portText: String) -> PortValidation {
        let trimmed = portText.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return .empty }
        guard let port = UInt16(trimmed), port >= minPort, port <= maxPort else {
            return .outOfRange
        }
        return port == forbiddenPort ? .forbidden : .valid
    }
}
