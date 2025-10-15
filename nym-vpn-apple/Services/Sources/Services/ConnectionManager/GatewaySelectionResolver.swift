import ConnectionTypes
import CountriesManagerTypes

actor GatewaySelectionResolver {
    func resolveEntryGateway(
        jsonString: String,
        connectionType: ConnectionType,
        entry: [GatewayNode],
        exit: [GatewayNode],
        vpn: [GatewayNode],
        entryCountries: [NymCountry],
        exitCountries: [NymCountry],
        vpnCountries: [NymCountry]
    ) -> EntryGateway {
        let nodeType: NodeType = connectionType == .wireguard ? .vpn : .entry

        guard let stored = EntryGateway.from(jsonString: jsonString)
        else {
            let fallback = fallbackCountry(
                for: nodeType,
                entryCountries: entryCountries,
                exitCountries: exitCountries,
                vpnCountries: vpnCountries
            )
            return .country(fallback.code)
        }

        switch stored {
        case let .country(code):
            let country = existingCountry(
                with: code,
                nodeType: nodeType,
                entryCountries: entryCountries,
                exitCountries: exitCountries,
                vpnCountries: vpnCountries
            )
            return .country(country.code)

        case let .lowLatencyCountry(code):
            let country = existingCountry(
                with: code,
                nodeType: nodeType,
                entryCountries: entryCountries,
                exitCountries: exitCountries,
                vpnCountries: vpnCountries
            )
            return .lowLatencyCountry(country.code)

        case let .gateway(identifier):
            if let gw = existingGateway(
                with: identifier,
                nodeType: nodeType,
                entry: entry,
                exit: exit,
                vpn: vpn
            ) {
                return .gateway(gw.id)
            } else {
                // Fall back to the gateway's country if we can infer it, otherwise to standard fallback
                let inferredCountry = countryForGateway(
                    id: identifier,
                    nodeType: nodeType,
                    entry: entry,
                    exit: exit,
                    vpn: vpn,
                    entryCountries: entryCountries,
                    exitCountries: exitCountries,
                    vpnCountries: vpnCountries
                ) ?? fallbackCountry(
                    for: nodeType,
                    entryCountries: entryCountries,
                    exitCountries: exitCountries,
                    vpnCountries: vpnCountries
                )

                let ensured = existingCountry(
                    with: inferredCountry.code,
                    nodeType: nodeType,
                    entryCountries: entryCountries,
                    exitCountries: exitCountries,
                    vpnCountries: vpnCountries
                )
                return .country(ensured.code)
            }

        case let .region(countryCode: code, region: region):
            return .region(countryCode: code, region: region)
        case let .city(city):
            return .city(city)
        case .random:
            return .random
        }
    }

    func resolveExitRouter(
        jsonString: String,
        connectionType: ConnectionType,
        entry: [GatewayNode],
        exit: [GatewayNode],
        vpn: [GatewayNode],
        entryCountries: [NymCountry],
        exitCountries: [NymCountry],
        vpnCountries: [NymCountry]
    ) -> ExitRouter {
        let nodeType: NodeType = connectionType == .wireguard ? .vpn : .exit

        guard let stored = ExitRouter.from(jsonString: jsonString)
        else {
            let fallback = fallbackCountry(
                for: nodeType,
                entryCountries: entryCountries,
                exitCountries: exitCountries,
                vpnCountries: vpnCountries
            )
            return .country(fallback.code)
        }

        switch stored {
        case let .country(code):
            let country = existingCountry(
                with: code,
                nodeType: nodeType,
                entryCountries: entryCountries,
                exitCountries: exitCountries,
                vpnCountries: vpnCountries
            )
            return .country(country.code)

        case let .gateway(identifier):
            if let gw = existingGateway(
                with: identifier,
                nodeType: nodeType,
                entry: entry,
                exit: exit,
                vpn: vpn
            ) {
                return .gateway(gw.id)
            } else {
                let inferredCountry = countryForGateway(
                    id: identifier,
                    nodeType: nodeType,
                    entry: entry,
                    exit: exit,
                    vpn: vpn,
                    entryCountries: entryCountries,
                    exitCountries: exitCountries,
                    vpnCountries: vpnCountries
                ) ?? fallbackCountry(
                    for: nodeType,
                    entryCountries: entryCountries,
                    exitCountries: exitCountries,
                    vpnCountries: vpnCountries
                )

                let ensured = existingCountry(
                    with: inferredCountry.code,
                    nodeType: nodeType,
                    entryCountries: entryCountries,
                    exitCountries: exitCountries,
                    vpnCountries: vpnCountries
                )
                return .country(ensured.code)
            }

        case let .address(address):
            return .address(address)

        case let .region(countryCode: code, region: region):
            return .region(countryCode: code, region: region)

        case .random:
            return .random
        }
    }

    private func existingCountry(
        with countryCode: String,
        nodeType: NodeType,
        entryCountries: [NymCountry],
        exitCountries: [NymCountry],
        vpnCountries: [NymCountry]
    ) -> NymCountry {
        switch nodeType {
        case .entry:
            if let country = entryCountries.first(
                where: { $0.code.caseInsensitiveCompare(countryCode) == .orderedSame }
            ) {
                return country
            }
        case .exit:
            if let country = exitCountries.first(
                where: { $0.code.caseInsensitiveCompare(countryCode) == .orderedSame }
            ) {
                return country
            }
        case .vpn:
            if let country = vpnCountries.first(
                where: { $0.code.caseInsensitiveCompare(countryCode) == .orderedSame }
            ) {
                return country
            }
        }
        return fallbackCountry(
            for: nodeType,
            entryCountries: entryCountries,
            exitCountries: exitCountries,
            vpnCountries: vpnCountries
        )
    }

    private func fallbackCountry(
        for nodeType: NodeType,
        entryCountries: [NymCountry],
        exitCountries: [NymCountry],
        vpnCountries: [NymCountry]
    ) -> NymCountry {
        let ch = NymCountry(name: "Switzerland", code: "CH", regions: [])
        switch nodeType {
        case .entry:
            if entryCountries.contains(where: { $0.code == "CH" }) {
                return ch
            }
            return entryCountries.first ?? ch
        case .exit:
            if exitCountries.contains(where: { $0.code == "CH" }) {
                return ch
            }
            return exitCountries.first ?? ch
        case .vpn:
            if vpnCountries.contains(where: { $0.code == "CH" }) {
                return ch
            }
            return vpnCountries.first ?? ch
        }
    }

    private func existingGateway(
        with gatewayId: String,
        nodeType: NodeType,
        entry: [GatewayNode],
        exit: [GatewayNode],
        vpn: [GatewayNode]
    ) -> GatewayNode? {
        switch nodeType {
        case .entry:
            return entry.first { $0.id == gatewayId }
        case .exit:
            return exit.first { $0.id == gatewayId }
        case .vpn:
            return vpn.first { $0.id == gatewayId }
        }
    }

    private func countryForGateway(
        id: String,
        nodeType: NodeType,
        entry: [GatewayNode],
        exit: [GatewayNode],
        vpn: [GatewayNode],
        entryCountries: [NymCountry],
        exitCountries: [NymCountry],
        vpnCountries: [NymCountry]
    ) -> NymCountry? {
        let node: GatewayNode?
        switch nodeType {
        case .entry:
            node = entry.first(where: { $0.id == id })
        case .exit:
            node = exit.first(where: { $0.id == id })
        case .vpn:
            node = vpn.first(where: { $0.id == id })
        }

        guard let code = node?.location?.twoLetterIsoCountryCode
        else {
            return nil
        }

        switch nodeType {
        case .entry:
            return entryCountries.first { $0.code.caseInsensitiveCompare(code) == .orderedSame }
        case .exit:
            return exitCountries.first { $0.code.caseInsensitiveCompare(code) == .orderedSame }
        case .vpn:
            return vpnCountries.first { $0.code.caseInsensitiveCompare(code) == .orderedSame }
        }
    }
}
