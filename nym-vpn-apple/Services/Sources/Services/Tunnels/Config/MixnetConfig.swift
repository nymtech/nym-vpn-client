import Foundation
import Network

public struct MixnetConfig: Codable {
    public enum EntryGateway: Codable {
        case location(String)
    }

    public enum ExitRouter: Codable {
        case location(String)
    }

    let apiUrl: String
    let explorerURL: String
    let entryGateway: EntryGateway
    let exitRouter: ExitRouter

    public var name = "NymVPN Mixnet"

    public init(
        apiUrl: String,
        explorerURL: String,
        entryGateway: EntryGateway,
        exitRouter: ExitRouter,
        name: String = "NymVPN Mixnet"
    ) {
        self.apiUrl = apiUrl
        self.explorerURL = explorerURL
        self.entryGateway = entryGateway
        self.exitRouter = exitRouter
        self.name = name
    }
}
