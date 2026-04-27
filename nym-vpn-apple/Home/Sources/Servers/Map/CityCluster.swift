import CoreLocation
import ConnectionTypes

struct CityCluster: Identifiable {
    let id: String
    let city: String
    let countryCode: String
    let coordinate: CLLocationCoordinate2D
    let nodeCount: Int
    let nodes: [GatewayNode]
}

extension CityCluster {
    static func clusters(from nodes: [GatewayNode]) -> [CityCluster] {
        let located = nodes.compactMap { node -> (GatewayNode, GatewayNodeLocation)? in
            guard let location = node.location else { return nil }
            return (node, location)
        }

        let grouped = Dictionary(grouping: located) { _, location in
            "\(location.city)-\(location.twoLetterIsoCountryCode)"
        }

        return grouped.map { key, entries in
            let city = entries[0].1.city
            let countryCode = entries[0].1.twoLetterIsoCountryCode
            let avgLat = entries.map(\.1.latitude).reduce(0, +) / Double(entries.count)
            let avgLon = entries.map(\.1.longitude).reduce(0, +) / Double(entries.count)
            let nodes = entries.map(\.0)

            return CityCluster(
                id: key,
                city: city,
                countryCode: countryCode,
                coordinate: CLLocationCoordinate2D(latitude: avgLat, longitude: avgLon),
                nodeCount: nodes.count,
                nodes: nodes
            )
        }
        .sorted { $0.city < $1.city }
    }
}
