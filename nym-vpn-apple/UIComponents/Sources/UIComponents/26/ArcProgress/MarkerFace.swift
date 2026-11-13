import SwiftUI

/// The connection marker's face (#7063) says the mode. dVPN: an entry and an
/// exit block, linked (2 hops). Mixnet: the three mix layers as a 3x3 field,
/// with the five hops lit (entry, one node per layer, exit); hollow nodes are
/// the unused ones. While not protected nothing is lit: dVPN shows only its
/// outline, Mixnet only its nodes. While connected the face plays an
/// illustrative route: a packet steps through the hops, and the Mixnet route
/// changes every two passes. It illustrates how the network works; it is not
/// a traffic meter. Drawn in an 80 x 80 box scaled to the frame. Decorative.
public struct MarkerFace: View {
    public let mode: MarkerMode
    /// false while not protected: no packets, only the outline or the nodes
    public let lit: Bool
    /// true while connected: plays the illustrative route
    public let playing: Bool
    public let litColor: Color
    public let unlitColor: Color

    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.scenePhase) private var scenePhase
    @State private var playStart = Date()
    @State private var isOnScreen = true

    public init(mode: MarkerMode, lit: Bool, playing: Bool, litColor: Color, unlitColor: Color) {
        self.mode = mode
        self.lit = lit
        self.playing = playing
        self.litColor = litColor
        self.unlitColor = unlitColor
    }

    public var body: some View {
        content
            .accessibilityHidden(true)
            .onAppear { isOnScreen = true }
            .onDisappear { isOnScreen = false }
            .onChange(of: isPlaying) { _, newValue in
                if newValue {
                    playStart = Date()
                }
            }
    }
}

private extension MarkerFace {
    /// The route plays only while it can be seen: the scene is active and the
    /// marker is on screen (#7063: the animation pauses when not visible).
    var isPlaying: Bool {
        lit && playing && !reduceMotion && scenePhase == .active && isOnScreen
    }

    @ViewBuilder
    var content: some View {
        if isPlaying {
            let interval = MarkerFaceModel.tickInterval(for: mode)
            let start = playStart
            TimelineView(.periodic(from: start, by: interval)) { context in
                canvas(tick: MarkerFaceModel.tick(from: start, to: context.date, interval: interval))
            }
        } else {
            canvas(tick: nil)
        }
    }

    func canvas(tick: Int?) -> some View {
        let mode = mode
        let lit = lit
        let color = lit ? litColor : unlitColor
        return Canvas { context, size in
            context.scaleBy(x: size.width / MarkerFaceModel.box, y: size.height / MarkerFaceModel.box)
            switch mode {
            case .dvpn:
                MarkerFaceModel.drawDvpn(in: context, color: color, lit: lit, tick: tick)
            case .mixnet:
                MarkerFaceModel.drawMixnet(in: context, color: color, lit: lit, tick: tick)
            }
        }
    }
}

/// Geometry and rhythm of the face, in the 80 x 80 design box.
enum MarkerFaceModel {
    static let box: CGFloat = 80
    static let strokeWidth: CGFloat = 2
    static let nodeStrokeWidth: CGFloat = 1.5

    static let dvpnEntry = CGRect(x: 13, y: 31, width: 18, height: 18)
    static let dvpnExit = CGRect(x: 49, y: 31, width: 18, height: 18)
    static let dvpnLink = (from: CGPoint(x: 31, y: 40), to: CGPoint(x: 49, y: 40))

    static let mixEntry = CGPoint(x: 8, y: 40)
    static let mixExit = CGPoint(x: 72, y: 40)
    static let layerX: [CGFloat] = [24, 40, 56]
    static let layerY: [CGFloat] = [22, 40, 58]
    static let nodeRadius: CGFloat = 3.5
    static let routeSquare: CGFloat = 10
    static let mixRoutes: [[CGPoint]] = [
        [mixEntry, CGPoint(x: 24, y: 22), CGPoint(x: 40, y: 58), CGPoint(x: 56, y: 22), mixExit],
        [mixEntry, CGPoint(x: 24, y: 40), CGPoint(x: 40, y: 22), CGPoint(x: 56, y: 40), mixExit],
        [mixEntry, CGPoint(x: 24, y: 58), CGPoint(x: 40, y: 22), CGPoint(x: 56, y: 58), mixExit],
        [mixEntry, CGPoint(x: 24, y: 40), CGPoint(x: 40, y: 58), CGPoint(x: 56, y: 40), mixExit]
    ]

    // rhythm: the spec's data line ticks (dVPN 80 ms, Mixnet 120 ms)
    static let dvpnTick: TimeInterval = 0.08
    static let mixTick: TimeInterval = 0.12
    /// entry, link, exit: 240 ms each
    static let dvpnTicksPerHop = 3
    /// 240 ms per hop
    static let mixTicksPerHop = 2
    /// five hops and one rest beat
    static let mixPass = 6
    static let mixPassesPerRoute = 2
    static let dim = 0.32
    static let trail = 0.6
    static let playingRouteLine = 0.4

    static func tickInterval(for mode: MarkerMode) -> TimeInterval {
        switch mode {
        case .dvpn:
            return dvpnTick
        case .mixnet:
            return mixTick
        }
    }

    /// Ticks elapsed since play started; the periodic timeline fires on them.
    static func tick(from start: Date, to now: Date, interval: TimeInterval) -> Int {
        max(0, Int((now.timeIntervalSince(start) / interval).rounded()))
    }

    /// dVPN lit element: 0 entry, 1 link, 2 exit; nil when still.
    static func dvpnHop(tick: Int?) -> Int? {
        tick.map { ($0 / dvpnTicksPerHop) % 3 }
    }

    static func dvpnOpacity(element: Int, tick: Int?) -> Double {
        guard let hop = dvpnHop(tick: tick) else {
            return 1
        }
        return hop == element ? 1 : dim
    }

    static func mixRouteIndex(tick: Int?) -> Int {
        guard let tick else {
            return 0
        }
        let step = tick / mixTicksPerHop
        return (step / (mixPass * mixPassesPerRoute)) % mixRoutes.count
    }

    /// Mixnet lit hop (0 to 4, 5 is the rest beat); nil when still.
    static func mixHop(tick: Int?) -> Int? {
        tick.map { ($0 / mixTicksPerHop) % mixPass }
    }

    /// Lit hop 1, passed hops `trail`, upcoming hops `dim`.
    static func mixHopOpacity(index: Int, tick: Int?) -> Double {
        guard let hop = mixHop(tick: tick) else {
            return 1
        }
        if index == hop {
            return 1
        }
        return index < hop ? trail : dim
    }

    static func drawDvpn(in context: GraphicsContext, color: Color, lit: Bool, tick: Int?) {
        let shading = GraphicsContext.Shading.color(color)
        let stroke = StrokeStyle(lineWidth: strokeWidth)

        var link = Path()
        link.move(to: dvpnLink.from)
        link.addLine(to: dvpnLink.to)
        var linkContext = context
        linkContext.opacity = dvpnOpacity(element: 1, tick: tick)
        linkContext.stroke(link, with: shading, style: stroke)

        for (element, rect) in [(0, dvpnEntry), (2, dvpnExit)] {
            var blockContext = context
            blockContext.opacity = dvpnOpacity(element: element, tick: tick)
            // one layer, so fill and stroke fade together
            blockContext.drawLayer { layer in
                let block = Path(rect)
                if lit {
                    layer.fill(block, with: shading)
                }
                layer.stroke(block, with: shading, style: stroke)
            }
        }
    }

    static func drawMixnet(in context: GraphicsContext, color: Color, lit: Bool, tick: Int?) {
        let shading = GraphicsContext.Shading.color(color)
        let stroke = StrokeStyle(lineWidth: strokeWidth)
        let route = mixRoutes[mixRouteIndex(tick: tick)]

        if lit {
            var line = Path()
            line.addLines(route)
            var lineContext = context
            lineContext.opacity = tick == nil ? 1 : playingRouteLine
            lineContext.stroke(line, with: shading, style: stroke)
        }

        var nodes: [CGPoint] = lit ? [] : [mixEntry, mixExit]
        for x in layerX {
            for y in layerY {
                let node = CGPoint(x: x, y: y)
                if !(lit && route.contains(node)) {
                    nodes.append(node)
                }
            }
        }
        for node in nodes {
            let circle = Path(ellipseIn: CGRect(
                x: node.x - nodeRadius,
                y: node.y - nodeRadius,
                width: nodeRadius * 2,
                height: nodeRadius * 2
            ))
            context.stroke(circle, with: shading, style: StrokeStyle(lineWidth: nodeStrokeWidth))
        }

        guard lit else {
            return
        }
        for (index, hop) in route.enumerated() {
            var squareContext = context
            squareContext.opacity = mixHopOpacity(index: index, tick: tick)
            squareContext.drawLayer { layer in
                let square = Path(CGRect(
                    x: hop.x - routeSquare / 2,
                    y: hop.y - routeSquare / 2,
                    width: routeSquare,
                    height: routeSquare
                ))
                layer.fill(square, with: shading)
                layer.stroke(square, with: shading, style: stroke)
            }
        }
    }
}
