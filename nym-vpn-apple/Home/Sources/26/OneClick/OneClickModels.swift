import Foundation
import SwiftUI
import AccountPrefetchGates
import ConnectionTypes
import Theme
import UIComponents

public enum OneClickConnectState: Equatable {
    case disconnected
    case connecting
    case stop
    case connected
    case disconnecting
    case noInternet
    case noAccount
    case noSubscription
    case accountUnreachable
    case checkingAccount

    static func disconnected(_ cta: DisconnectedHomeCTA) -> Self {
        switch cta {
        case .getStarted:
            .noAccount
        case .choosePlan:
            .noSubscription
        case .accountUnreachable:
            .accountUnreachable
        case .checking:
            .checkingAccount
        case .connect:
            .disconnected
        }
    }
}

public enum OneClickDisplayMode: String, Codable, Equatable, Sendable, CaseIterable {
    case powerUser
    case nerd
}

public enum OneClickSpeedMode: String, Codable, Equatable, Sendable, CaseIterable {
    case fast
    case anonymous
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
    public var gateway: GatewayNode?
    public var hopType: HopType?
    public var showsInfoButton: Bool
    public var isRandomSelection: Bool
    public var isSafestSelection: Bool

    public init(
        countryCode: String,
        title: String,
        subtitle: String?,
        score: OneClickServerScore,
        gateway: GatewayNode? = nil,
        hopType: HopType? = nil,
        showsInfoButton: Bool = false,
        isRandomSelection: Bool = false,
        isSafestSelection: Bool = false
    ) {
        self.countryCode = countryCode
        self.title = title
        self.subtitle = subtitle
        self.score = score
        self.gateway = gateway
        self.hopType = hopType
        self.showsInfoButton = showsInfoButton
        self.isRandomSelection = isRandomSelection
        self.isSafestSelection = isSafestSelection
    }
}

public enum OneClickSelectionPhase: Equatable {
    case selecting
    case selected(OneClickServerInfo)

    var selectedInfo: OneClickServerInfo? {
        if case let .selected(info) = self { return info }
        return nil
    }
}
