import SwiftUI

/// dVPN: entry and exit blocks, linked. Mixnet: three mix layers as a 3x3 field with the five hops of a route lit.
/// Decorative. `lit` is true only while connected; `playing` steps a packet through the hops.
public struct MarkerFace: View {
    public let mode: ArcProgressMode
    public let lit: Bool
    public let playing: Bool
    public let litColor: Color
    public let unlitColor: Color

    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.scenePhase) private var scenePhase
    @State private var playStart = Date()
    @State private var isOnScreen = true

    public init(mode: ArcProgressMode, lit: Bool, playing: Bool, litColor: Color, unlitColor: Color) {
        self.mode = mode
        self.lit = lit
        self.playing = playing
        self.litColor = litColor
        self.unlitColor = unlitColor
    }

    public var body: some View {
        // the hooks sit on a stable ZStack, not on the branches, so a branch swap cannot flip isOnScreen
        ZStack { content }
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
    /// Plays only while visible: scene active and on screen.
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
            case .fast:
                MarkerFaceModel.drawDvpn(in: context, color: color, lit: lit, tick: tick)
            case .anonymous:
                MarkerFaceModel.drawMixnet(in: context, color: color, lit: lit, tick: tick)
            }
        }
    }
}

/// Geometry and rhythm of the face, in an 80 x 80 box.
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

    static let dvpnTick: TimeInterval = 0.08
    static let mixTick: TimeInterval = 0.12
    static let dvpnTicksPerHop = 3
    /// three hops and two rest beats (480 ms pause)
    static let dvpnPass = 5
    static let mixTicksPerHop = 2
    /// five hops and one rest beat
    static let mixPass = 6
    static let mixPassesPerRoute = 2
    static let dim = 0.32
    static let trail = 0.6
    static let playingRouteLine = 0.4

    static func tickInterval(for mode: ArcProgressMode) -> TimeInterval {
        switch mode {
        case .fast:
            return dvpnTick
        case .anonymous:
            return mixTick
        }
    }

    /// Ticks elapsed since play started; the periodic timeline fires on them.
    static func tick(from start: Date, to now: Date, interval: TimeInterval) -> Int {
        max(0, Int((now.timeIntervalSince(start) / interval).rounded()))
    }

    /// 0 entry, 1 link, 2 exit, 3 and up the rest beats (all dim); nil when still.
    static func dvpnHop(tick: Int?) -> Int? {
        tick.map { ($0 / dvpnTicksPerHop) % dvpnPass }
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
