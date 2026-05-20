import Foundation
import SwiftUI
import Theme

public enum OneClickConnectState: Equatable {
    case disconnected
    case connecting
    case stop
    case connected
    case disconnecting
    case noInternet
    case noSubscription
}

public enum OneClickDisplayMode: String, Codable, Equatable, Sendable, CaseIterable {
    case nerd
    case oneClick
    case powerUser
}

public enum OneClickServerScore: Equatable {
    case high
    case medium
    case low
    case offline

    var variableValue: Double {
        switch self {
        case .offline:
            0.0
        case .low:
            0.25
        case .medium:
            0.75
        case .high:
            1.0
        }
    }

    var activeColor: Color {
        switch self {
        case .high:
                .green
        case .medium:
                .yellow
        case .low:
                .red
        case .offline:
            Color.Nym.textTertiary
        }
    }
}

public struct OneClickServerInfo: Equatable {
    public var countryCode: String
    public var title: String
    public var subtitle: String?
    public var score: OneClickServerScore
    public var supportsPostQuantum: Bool
}

public enum OneClickSelectionPhase: Equatable {
    case selecting
    case selected(OneClickServerInfo)

    var selectedInfo: OneClickServerInfo? {
        if case let .selected(info) = self { return info }
        return nil
    }
}
