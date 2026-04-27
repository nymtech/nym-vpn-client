import SwiftUI
import MapKit
import Theme
import ConnectionTypes

public struct GatewayMapView: View {
    private let clusters: [CityCluster]
    private let onSelectCity: (String, String, [GatewayNode]) -> Void

    @State private var selectedCluster: CityCluster?

    public init(
        nodes: [GatewayNode],
        onSelectCity: @escaping (String, String, [GatewayNode]) -> Void
    ) {
        self.clusters = CityCluster.clusters(from: nodes)
        self.onSelectCity = onSelectCity
    }

    public var body: some View {
        GatewayMapRepresentable(
            clusters: clusters,
            selectedCluster: $selectedCluster
        )
        .overlay(alignment: .bottom) {
            if let cluster = selectedCluster {
                CityPopoverView(cluster: cluster) {
                    onSelectCity(cluster.city, cluster.countryCode, cluster.nodes)
                    selectedCluster = nil
                }
                .padding(.bottom, 40)
                .transition(.move(edge: .bottom).combined(with: .opacity))
            }
        }
        .edgesIgnoringSafeArea(.all)
    }
}

#if os(iOS)
private typealias PlatformColor = UIColor
#else
private typealias PlatformColor = NSColor
#endif

// MARK: - Basemap palette (from overlay-style.json)

private enum NymBasemap {
    static let landFill = PlatformColor(red: 0.180, green: 0.227, blue: 0.247, alpha: 1)
    static let waterFill = PlatformColor(red: 0.110, green: 0.106, blue: 0.133, alpha: 1)
    static let countryTitle = "nym.country"
}

// MARK: - Water tile overlay
//
// `MKTileOverlay` with `canReplaceMapContent = true` makes MapKit skip the
// standard basemap entirely — no flash of Apple's labels/colors during fast
// pans. We synthesize a single-colour PNG once and return it for every tile.

private final class WaterTileOverlay: MKTileOverlay {
    private let tileData: Data

    init() {
        self.tileData = WaterTileOverlay.makeTilePNG(color: NymBasemap.waterFill)
        super.init(urlTemplate: nil)
        self.canReplaceMapContent = true
        self.tileSize = CGSize(width: 256, height: 256)
    }

    override func loadTile(at path: MKTileOverlayPath, result: @escaping (Data?, Error?) -> Void) {
        result(tileData, nil)
    }

    override func url(forTilePath path: MKTileOverlayPath) -> URL {
        URL(string: "about:blank")!
    }

    private static func makeTilePNG(color: PlatformColor) -> Data {
        let size = CGSize(width: 8, height: 8)
        #if os(iOS)
        let renderer = UIGraphicsImageRenderer(size: size)
        let image = renderer.image { ctx in
            color.setFill()
            ctx.fill(CGRect(origin: .zero, size: size))
        }
        return image.pngData() ?? Data()
        #else
        let image = NSImage(size: size, flipped: false) { rect in
            color.setFill()
            rect.fill()
            return true
        }
        guard let tiff = image.tiffRepresentation,
              let rep = NSBitmapImageRep(data: tiff),
              let png = rep.representation(using: .png, properties: [:]) else {
            return Data()
        }
        return png
        #endif
    }
}

// MARK: - GeoJSON country loader

private enum NymCountryOverlay {
    static func loadPolygons() -> [MKPolygon] {
        guard let url = Bundle.module.url(forResource: "nym-countries", withExtension: "geojson") else {
            return []
        }
        do {
            let data = try Data(contentsOf: url)
            let features = try MKGeoJSONDecoder().decode(data).compactMap { $0 as? MKGeoJSONFeature }
            var polygons: [MKPolygon] = []
            for feature in features {
                for geometry in feature.geometry {
                    if let polygon = geometry as? MKPolygon {
                        appendIfValid(polygon, into: &polygons)
                    } else if let multi = geometry as? MKMultiPolygon {
                        for polygon in multi.polygons {
                            appendIfValid(polygon, into: &polygons)
                        }
                    }
                }
            }
            return polygons
        } catch {
            return []
        }
    }

    /// Skip rings that cross the antimeridian — MapKit renders them as
    /// world-spanning horizontal bars instead of wrapping. Detect via
    /// boundingMapRect width relative to the world width.
    private static func appendIfValid(_ polygon: MKPolygon, into polygons: inout [MKPolygon]) {
        let worldWidth = MKMapRect.world.size.width
        guard polygon.boundingMapRect.size.width < worldWidth * 0.9 else { return }
        polygon.title = NymBasemap.countryTitle
        polygons.append(polygon)
    }
}

// MARK: - MKMapView wrapper

private func installBasemapOverlays(on mapView: MKMapView) {
    mapView.addOverlay(WaterTileOverlay(), level: .aboveLabels)
    for polygon in NymCountryOverlay.loadPolygons() {
        mapView.addOverlay(polygon, level: .aboveLabels)
    }
}

#if os(iOS)
private struct GatewayMapRepresentable: UIViewRepresentable {
    let clusters: [CityCluster]
    @Binding var selectedCluster: CityCluster?

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    func makeUIView(context: Context) -> MKMapView {
        let mapView = MKMapView()
        mapView.delegate = context.coordinator
        mapView.overrideUserInterfaceStyle = .dark
        mapView.backgroundColor = NymBasemap.waterFill
        mapView.showsCompass = false
        mapView.showsBuildings = false
        mapView.showsTraffic = false
        mapView.preferredConfiguration = {
            let config = MKStandardMapConfiguration(elevationStyle: .flat, emphasisStyle: .muted)
            config.pointOfInterestFilter = .excludingAll
            config.showsTraffic = false
            return config
        }()

        installBasemapOverlays(on: mapView)
        addAnnotations(to: mapView)
        fitMap(mapView)
        return mapView
    }

    func updateUIView(_ mapView: MKMapView, context: Context) {
        context.coordinator.parent = self
    }

    private func addAnnotations(to mapView: MKMapView) {
        for cluster in clusters {
            let annotation = CityAnnotation(cluster: cluster)
            mapView.addAnnotation(annotation)
        }
    }

    private func fitMap(_ mapView: MKMapView) {
        guard !clusters.isEmpty else { return }
        let coordinates = clusters.map(\.coordinate)
        var rect = MKMapRect.null
        for coord in coordinates {
            let point = MKMapPoint(coord)
            rect = rect.union(MKMapRect(origin: point, size: MKMapSize(width: 1, height: 1)))
        }
        mapView.setVisibleMapRect(rect, edgePadding: .init(top: 40, left: 40, bottom: 40, right: 40), animated: false)
    }
}
#else
private struct GatewayMapRepresentable: NSViewRepresentable {
    let clusters: [CityCluster]
    @Binding var selectedCluster: CityCluster?

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    func makeNSView(context: Context) -> MKMapView {
        let mapView = MKMapView()
        mapView.delegate = context.coordinator
        mapView.appearance = NSAppearance(named: .darkAqua)
        mapView.wantsLayer = true
        mapView.layer?.backgroundColor = NymBasemap.waterFill.cgColor
        mapView.showsCompass = false
        mapView.showsBuildings = false
        mapView.showsTraffic = false
        mapView.preferredConfiguration = {
            let config = MKStandardMapConfiguration(elevationStyle: .flat, emphasisStyle: .muted)
            config.pointOfInterestFilter = .excludingAll
            config.showsTraffic = false
            return config
        }()

        installBasemapOverlays(on: mapView)
        addAnnotations(to: mapView)
        fitMap(mapView)
        return mapView
    }

    func updateNSView(_ mapView: MKMapView, context: Context) {
        context.coordinator.parent = self
    }

    private func addAnnotations(to mapView: MKMapView) {
        for cluster in clusters {
            let annotation = CityAnnotation(cluster: cluster)
            mapView.addAnnotation(annotation)
        }
    }

    private func fitMap(_ mapView: MKMapView) {
        guard !clusters.isEmpty else { return }
        let coordinates = clusters.map(\.coordinate)
        var rect = MKMapRect.null
        for coord in coordinates {
            let point = MKMapPoint(coord)
            rect = rect.union(MKMapRect(origin: point, size: MKMapSize(width: 1, height: 1)))
        }
        mapView.setVisibleMapRect(rect, edgePadding: .init(top: 40, left: 40, bottom: 40, right: 40), animated: false)
    }
}
#endif

// MARK: - Annotation model

private final class CityAnnotation: NSObject, MKAnnotation {
    let cluster: CityCluster
    var coordinate: CLLocationCoordinate2D { cluster.coordinate }
    var title: String? { cluster.city }

    init(cluster: CityCluster) {
        self.cluster = cluster
    }
}

// MARK: - Coordinator

private final class Coordinator: NSObject, MKMapViewDelegate {
    var parent: GatewayMapRepresentable

    init(parent: GatewayMapRepresentable) {
        self.parent = parent
    }

    func mapView(_ mapView: MKMapView, rendererFor overlay: MKOverlay) -> MKOverlayRenderer {
        if let tile = overlay as? MKTileOverlay {
            return MKTileOverlayRenderer(tileOverlay: tile)
        }
        if let polygon = overlay as? MKPolygon, polygon.title == NymBasemap.countryTitle {
            let renderer = MKPolygonRenderer(polygon: polygon)
            renderer.fillColor = NymBasemap.landFill
            renderer.strokeColor = .clear
            renderer.lineWidth = 0
            return renderer
        }
        return MKOverlayRenderer(overlay: overlay)
    }

    func mapView(_ mapView: MKMapView, viewFor annotation: MKAnnotation) -> MKAnnotationView? {
        guard let cityAnnotation = annotation as? CityAnnotation else { return nil }

        let identifier = "CityCluster"
        let view = mapView.dequeueReusableAnnotationView(withIdentifier: identifier)
            ?? MKAnnotationView(annotation: cityAnnotation, reuseIdentifier: identifier)

        view.annotation = cityAnnotation
        view.canShowCallout = false

        let dotSize: CGFloat = 10
        let glowSize: CGFloat = 18
        let accent = NymColor.accent

        view.image = Self.dotImage(dotSize: dotSize, glowSize: glowSize, accent: accent)
        view.centerOffset = .zero

        return view
    }

    #if os(iOS)
    private static func dotImage(dotSize: CGFloat, glowSize: CGFloat, accent: Color) -> UIImage {
        let renderer = UIGraphicsImageRenderer(size: CGSize(width: glowSize, height: glowSize))
        return renderer.image { ctx in
            let accentCG = UIColor(accent).cgColor
            UIColor(cgColor: accentCG.copy(alpha: 0.4) ?? accentCG).setFill()
            ctx.cgContext.fillEllipse(in: CGRect(x: 0, y: 0, width: glowSize, height: glowSize))

            UIColor(accent).setFill()
            let inset = (glowSize - dotSize) / 2
            ctx.cgContext.fillEllipse(in: CGRect(x: inset, y: inset, width: dotSize, height: dotSize))
        }
    }
    #else
    private static func dotImage(dotSize: CGFloat, glowSize: CGFloat, accent: Color) -> NSImage {
        let image = NSImage(size: NSSize(width: glowSize, height: glowSize), flipped: false) { rect in
            let accentCG = NSColor(accent).cgColor
            NSColor(cgColor: accentCG.copy(alpha: 0.4) ?? accentCG)?.setFill()
            NSBezierPath(ovalIn: rect).fill()

            NSColor(accent).setFill()
            let inset = (glowSize - dotSize) / 2
            NSBezierPath(ovalIn: CGRect(x: inset, y: inset, width: dotSize, height: dotSize)).fill()
            return true
        }
        return image
    }
    #endif

    func mapView(_ mapView: MKMapView, didSelect view: MKAnnotationView) {
        guard let cityAnnotation = view.annotation as? CityAnnotation else { return }
        withAnimation(.easeInOut(duration: 0.2)) {
            parent.selectedCluster = cityAnnotation.cluster
        }
        mapView.deselectAnnotation(view.annotation, animated: false)
    }
}
