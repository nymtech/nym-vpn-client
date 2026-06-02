import Foundation

public struct FoundApp {
    public let name: String
    public let executablePath: String?
    public let icon: URL?

    public init(name: String, executablePath: String?, icon: URL?) {
        self.name = name
        self.executablePath = executablePath
        self.icon = icon
    }
}
