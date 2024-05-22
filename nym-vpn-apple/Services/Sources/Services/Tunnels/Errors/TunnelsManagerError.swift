import Foundation

public enum TunnelsManagerError: Error {
    case emptyName
    case alreadyExists
    case tunnelList(error: Error)
    case addTunnel(error: Error)
    case modifyTunnel(error: Error)
    case removeTunnel(error: Error)
}
