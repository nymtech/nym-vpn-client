import Combine
import Foundation
import SwiftUI
import SnackbarManager
import AccountPrefetchGates
import AppSettings
import ConnectionManager
import ConnectionTypes
import CredentialsManager
import ErrorReason
import GatewayManager
import ImpactGenerator
import NetworkMonitor
import TunnelStatus
import UIComponents
#if os(macOS)
import GRPCManager
#endif

extension OneClickViewModel {
    func refreshSelection() {
        let cfg = connectionManager.connectionConfig
        let twoHop = cfg.enableTwoHop
        let exitType: NodeType = twoHop ? .vpn : .exit
        let entryType: NodeType = twoHop ? .vpn : .entry
        let info = connectionManager.connectionInfoData
        let liveCandidate = isLiveStatus(connectionManager.currentTunnelStatus) && info != nil
        resolveTask?.cancel()
        resolveTask = Task { @MainActor [weak self] in
            self?.resolveSelection(
                cfg: cfg,
                entryType: entryType,
                exitType: exitType,
                info: info,
                liveCandidate: liveCandidate
            )
        }
    }

    func resolveSelection(
        cfg: ConnectionConfig,
        entryType: NodeType,
        exitType: NodeType,
        info: ConnectionInfoData?,
        liveCandidate: Bool
    ) {
        if liveCandidate,
           let info,
           let exitLive = livePhase(
               gatewayId: info.exitGatewayId,
               gatewayType: exitType,
               hopType: .exit,
               displayMode: displayMode
           ) {
            isLiveConnection = true
            selectionPhase = exitLive
            if displayMode == .nerd {
                entrySelectionPhase = livePhase(
                    gatewayId: info.entryGatewayId,
                    gatewayType: entryType,
                    hopType: .entry,
                    displayMode: displayMode
                )
                    ?? resolveEntryPhase(
                        entry: cfg.entry,
                        gatewayId: info.entryGatewayId ?? cfg.entry.gatewayId,
                        gatewayType: entryType
                    )
            } else {
                entrySelectionPhase = .selecting
            }
            return
        }
        isLiveConnection = false
        switch displayMode {
        case .powerUser:
            entrySelectionPhase = .selecting
            selectionPhase = resolveExitPhase(
                exit: cfg.exit,
                gatewayId: cfg.exit.gatewayId,
                gatewayType: exitType
            )
        case .nerd:
            entrySelectionPhase = resolveEntryPhase(
                entry: cfg.entry,
                gatewayId: cfg.entry.gatewayId,
                gatewayType: entryType
            )
            selectionPhase = resolveExitPhase(
                exit: cfg.exit,
                gatewayId: cfg.exit.gatewayId,
                gatewayType: exitType
            )
        }
    }

    func isLiveStatus(_ status: TunnelStatus) -> Bool {
        switch status {
        case .connecting, .reasserting, .restarting, .connected, .offlineReconnect, .disconnecting:
            return true
        case .disconnected, .offline, .error, .unknown:
            return false
        }
    }

    func livePhase(
        gatewayId: String?,
        gatewayType: NodeType,
        hopType: HopType,
        displayMode: OneClickDisplayMode
    ) -> OneClickSelectionPhase? {
        guard let gatewayId,
              let gateway = gatewayManager.gateway(with: gatewayId, gatewayType: gatewayType)
        else { return nil }
        let title: String
        switch displayMode {
        case .nerd:
            title = gateway.name ?? gateway.id
        case .powerUser:
            title = gateway.ipv4s.first ?? gateway.name ?? gateway.id
        }
        let location = gateway.location
        let subtitle: String?
        if let location {
            let country = gatewayManager.localizedCountry(with: location.twoLetterIsoCountryCode)?.name
            if gatewayManager.shouldDisplayRegion(with: location.twoLetterIsoCountryCode) {
                subtitle = "\(location.city), \(location.region), \(country ?? "")"
            } else {
                subtitle = "\(location.city), \(country ?? "")"
            }
        } else {
            subtitle = nil
        }
        return .selected(OneClickServerInfo(
            countryCode: location?.twoLetterIsoCountryCode ?? "",
            title: title,
            subtitle: subtitle,
            score: score(for: gateway),
            gateway: gateway,
            hopType: hopType,
            showsInfoButton: true
        ))
    }

    func resolveEntryPhase(entry: EntryGateway, gatewayId: String?, gatewayType: NodeType) -> OneClickSelectionPhase {
        let gateway = gatewayId.flatMap { gatewayManager.gateway(with: $0, gatewayType: gatewayType) }
        let title: String
        if case .random = entry {
            title = "gatewaysView.random".localizedString
        } else if case .auto = entry {
            title = "gatewaysView.safest".localizedString
        } else {
            title = gatewayManager.userFriendlyTitle(with: entry)
                ?? gateway?.location?.city ?? gateway?.name ?? gateway?.id ?? ""
        }
        let subtitle = entrySubtitle(entry: entry, gateway: gateway)
        let countryCode = gatewayManager.countryCode(with: entry)
            ?? gateway?.location?.twoLetterIsoCountryCode ?? ""
        let scoreGateway = scoreGateway(forEntry: entry, fallback: gateway, gatewayType: gatewayType)
        let isExplicitGateway: Bool
        if case .gateway = entry { isExplicitGateway = true } else { isExplicitGateway = false }
        let isRandomSelection: Bool
        if case .random = entry { isRandomSelection = true } else { isRandomSelection = false }
        let isSafestSelection: Bool
        if case .auto = entry { isSafestSelection = true } else { isSafestSelection = false }
        return makePhase(
            title: title,
            subtitle: subtitle,
            countryCode: countryCode,
            scoreGateway: scoreGateway,
            hopType: .entry,
            showsInfoButton: isExplicitGateway,
            showsScore: !(isRandomSelection || isSafestSelection),
            isRandomSelection: isRandomSelection,
            isSafestSelection: isSafestSelection
        )
    }

    func resolveExitPhase(exit: ExitRouter, gatewayId: String?, gatewayType: NodeType) -> OneClickSelectionPhase {
        let gateway = gatewayId.flatMap { gatewayManager.gateway(with: $0, gatewayType: gatewayType) }
        let title: String
        if case .random = exit {
            title = "gatewaysView.random".localizedString
        } else if case .auto = exit {
            title = "gatewaysView.safest".localizedString
        } else {
            title = gatewayManager.userFriendlyTitle(with: exit)
                ?? gateway?.location?.city ?? gateway?.name ?? gateway?.id ?? ""
        }
        let subtitle = exitSubtitle(exit: exit, gateway: gateway)
        let countryCode = gatewayManager.countryCode(with: exit)
            ?? gateway?.location?.twoLetterIsoCountryCode ?? ""
        let scoreGateway = scoreGateway(forExit: exit, fallback: gateway, gatewayType: gatewayType)
        let isExplicitGateway: Bool
        if case .gateway = exit { isExplicitGateway = true } else { isExplicitGateway = false }
        let isRandomSelection: Bool
        if case .random = exit { isRandomSelection = true } else { isRandomSelection = false }
        let isSafestSelection: Bool
        if case .auto = exit { isSafestSelection = true } else { isSafestSelection = false }
        return makePhase(
            title: title,
            subtitle: subtitle,
            countryCode: countryCode,
            scoreGateway: scoreGateway,
            hopType: .exit,
            showsInfoButton: isExplicitGateway,
            showsScore: !(isRandomSelection || isSafestSelection),
            isRandomSelection: isRandomSelection,
            isSafestSelection: isSafestSelection
        )
    }

    func scoreGateway(forEntry entry: EntryGateway, fallback: GatewayNode?, gatewayType: NodeType) -> GatewayNode? {
        switch entry {
        case .country, .region, .random, .auto:
            return gatewayManager.bestGateway(matching: entry, gatewayType: gatewayType) ?? fallback
        case .gateway:
            return fallback
        }
    }

    func scoreGateway(forExit exit: ExitRouter, fallback: GatewayNode?, gatewayType: NodeType) -> GatewayNode? {
        switch exit {
        case .country, .region, .random, .auto:
            return gatewayManager.bestGateway(matching: exit, gatewayType: gatewayType) ?? fallback
        case .gateway:
            return fallback
        }
    }

    func makePhase(
        title: String,
        subtitle: String?,
        countryCode: String,
        scoreGateway: GatewayNode?,
        hopType: HopType,
        showsInfoButton: Bool = false,
        showsScore: Bool = true,
        isRandomSelection: Bool = false,
        isSafestSelection: Bool = false
    ) -> OneClickSelectionPhase {
        if title.isEmpty && (subtitle?.isEmpty ?? true) {
            return .selecting
        }
        return .selected(OneClickServerInfo(
            countryCode: countryCode,
            title: title,
            subtitle: subtitle,
            score: scoreGateway.map { score(for: $0) } ?? .offline,
            gateway: scoreGateway,
            hopType: hopType,
            showsInfoButton: showsInfoButton,
            showsScore: showsScore,
            isRandomSelection: isRandomSelection,
            isSafestSelection: isSafestSelection
        ))
    }

    func score(for gateway: GatewayNode) -> OneClickServerScore {
        let raw: GatewayNodeScore?
        switch connectionManager.connectionType {
        case .wireguard:
            raw = gateway.performance?.score
        case .mixnet5hop:
            raw = gateway.performance?.mixnetScore ?? gateway.mixnetScore
        }
        return Self.score(from: raw ?? .offline)
    }

    func entrySubtitle(entry: EntryGateway, gateway: GatewayNode?) -> String? {
        guard let location = gateway?.location else { return nameOrId(gateway: gateway) }
        switch entry {
        case let .country(code):
            return countrySubtitle(gateway: gateway, countryCode: code, location: location)
        case .region:
            return regionSubtitle(gateway: gateway, location: location)
        case .gateway:
            return serverSubtitle(location: location, countryCode: location.twoLetterIsoCountryCode)
        case .random, .auto:
            return nil
        }
    }

    func exitSubtitle(exit: ExitRouter, gateway: GatewayNode?) -> String? {
        guard let location = gateway?.location else { return nameOrId(gateway: gateway) }
        switch exit {
        case let .country(code):
            return countrySubtitle(gateway: gateway, countryCode: code, location: location)
        case .region:
            return regionSubtitle(gateway: gateway, location: location)
        case .gateway:
            return serverSubtitle(location: location, countryCode: location.twoLetterIsoCountryCode)
        case .random, .auto:
            return nil
        }
    }

    func countrySubtitle(gateway: GatewayNode?, countryCode: String, location: GatewayNodeLocation) -> String? {
        if gatewayManager.shouldDisplayRegion(with: countryCode) {
            "\(location.city), \(location.region) \(nameOrId(gateway: gateway))"
        } else {
            "\(location.city) \(nameOrId(gateway: gateway))"
        }
    }

    func regionSubtitle(gateway: GatewayNode?, location: GatewayNodeLocation) -> String? {
        "\(location.city) \(nameOrId(gateway: gateway))"
    }

    func serverSubtitle(location: GatewayNodeLocation, countryCode: String) -> String? {
        let country = gatewayManager.localizedCountry(with: countryCode)
        if gatewayManager.shouldDisplayRegion(with: countryCode) {
            return "\(location.city), \(location.region), \(country?.name ?? "")"
        } else {
            return "\(location.city), \(country?.name ?? "")"
        }
    }

    func nameOrId(gateway: GatewayNode?) -> String {
        if let name = gateway?.name {
            "(\(name))"
        } else if let identifier = gateway?.id {
            "(\(identifier))"
        } else {
            ""
        }
    }

    static func score(from node: GatewayNodeScore) -> OneClickServerScore {
        switch node {
        case .high:
            return .high
        case .medium:
            return .medium
        case .low:
            return .low
        case .offline, .noScore:
            return .offline
        }
    }
}
