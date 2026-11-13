import SwiftUI
import Theme

/// SwiftUI port of the Figma "Arc Progress" component (node `1012:22864`).
///
/// Three concentric arcs animate the six-step VPN connection sequence using a
/// half-sweep pattern: outer ring covers steps 1–2, middle ring 3–4, inner
/// ring 5–6. A radial sphere sits at the centre with a glow that ramps in
/// when the connection is established.
public struct ArcProgressView: View {
    public let state: ArcProgressState
    public let mode: ArcProgressMode
    public let markerMode: MarkerMode
    public let connectedDate: Date?
    public let showsIndependenceWarning: Bool
    public let availableHeight: CGFloat?

    @State private var lastStep: ArcProgressState.Step?
    @Environment(\.colorScheme) private var colorScheme

    public init(
        state: ArcProgressState,
        mode: ArcProgressMode = .fast,
        markerMode: MarkerMode = .dvpn,
        connectedDate: Date? = nil,
        showsIndependenceWarning: Bool = false,
        availableHeight: CGFloat? = nil
    ) {
        self.state = state
        self.mode = mode
        self.markerMode = markerMode
        self.connectedDate = connectedDate
        self.showsIndependenceWarning = showsIndependenceWarning
        self.availableHeight = availableHeight
    }

    public var body: some View {
        VStack(spacing: Constants.labelTopSpacing) {
            arcStack
                .frame(width: Constants.canvasSize, height: Constants.canvasSize)

            Text(labelText)
                .font(.system(size: Constants.labelFontSize))
                .foregroundColor(labelColor)
                .frame(minHeight: Constants.labelMinHeight)
                .animation(.easeOut(duration: 0.3), value: state)
                .animation(.easeInOut(duration: 0.2), value: labelColor)
        }
        .overlay(alignment: .top) {
            timerOverlay
        }
        .overlay(alignment: .bottom) {
            secondaryWarningOverlay
        }
        .scaleEffect(contentScale, anchor: .center)
        .animation(.easeInOut(duration: 0.2), value: contentScale)
        .onAppear { recordStepIfActive(state) }
        .onChange(of: state) { _, newValue in
            recordStepIfActive(newValue)
        }
    }
}

private extension ArcProgressView {
    var arcStack: some View {
        ZStack {
            ring(.outer)
            ring(.middle)
            ring(.inner)
            glow
            sphere
            face
        }
    }

    /// The mode face, centred in the rings: unlit in the track grey while not
    /// protected, playing the illustrative route while connected.
    var face: some View {
        MarkerFace(
            mode: markerMode,
            lit: markerState != .notProtected,
            playing: markerState == .connected,
            litColor: Color.Nym.textPrimary,
            unlitColor: trackColor
        )
        .frame(width: Constants.sphereDiameter, height: Constants.sphereDiameter)
    }

    func ring(_ ring: Ring) -> some View {
        ZStack {
            Circle()
                .stroke(trackColor, style: ringStroke(for: ring))
                .frame(width: ring.diameter, height: ring.diameter)
                .rotationEffect(.degrees(-90))

            Circle()
                .trim(from: 0, to: progress(for: ring))
                .stroke(fillColor, style: ringStroke(for: ring))
                .frame(width: ring.diameter, height: ring.diameter)
                .rotationEffect(.degrees(-90))
                .opacity(fillOpacity)
                .animation(sweepAnimation, value: progress(for: ring))
                .animation(.easeIn(duration: 0.6), value: fillOpacity)
                .animation(.easeOut(duration: 0.2), value: fillColor)
                .modifier(MarkerBlink(isActive: markerState == .blocked && ring == .outer))
        }
    }

    var sphereFill: AnyShapeStyle {
        if colorScheme == .dark {
            AnyShapeStyle(
                RadialGradient(
                    colors: [
                        Color.Nym.surfaceAlt,
                        Color.Nym.background
                    ],
                    center: UnitPoint(x: 0.3, y: 0.3),
                    startRadius: 0,
                    endRadius: Constants.sphereDiameter * 0.8
                )
            )
        } else {
            AnyShapeStyle(Color.black.opacity(0.42))
        }
    }

    var sphere: some View {
        Circle()
            .fill(sphereFill)
            .overlay(
                Circle()
                    .fill(Constants.errorTint)
                    .opacity(state == .failed ? 1 : 0)
            )
            .frame(width: Constants.sphereDiameter, height: Constants.sphereDiameter)
            .opacity(sphereOpacity)
            .shadow(
                color: haloColor,
                radius: state == .connected ? haloRadius : 0
            )
            .animation(.easeOut(duration: 0.3), value: sphereOpacity)
            .animation(.easeOut(duration: 0.32), value: haloColor)
    }

    var glow: some View {
        Circle()
            .fill(
                RadialGradient(
                    colors: [fillColor, .clear],
                    center: .center,
                    startRadius: 0,
                    endRadius: Constants.glowDiameter / 2
                )
            )
            .frame(width: Constants.glowDiameter, height: Constants.glowDiameter)
            .opacity(state == .connected ? Constants.glowConnectedOpacity : 0)
            .animation(.easeOut(duration: 0.32), value: state)
    }

    @ViewBuilder
    var secondaryWarningOverlay: some View {
        if let secondaryCaptionKey {
            Text(secondaryCaptionKey.localizedString)
                .font(.system(size: Constants.warningFontSize))
                .foregroundColor(Color.Nym.textTertiary)
                .multilineTextAlignment(.center)
                .fixedSize(horizontal: false, vertical: true)
                .frame(width: Constants.canvasSize)
                .offset(y: Constants.warningTopOffset)
                .transition(.opacity)
                .animation(.easeInOut(duration: 0.25), value: secondaryCaptionKey)
        }
    }

    @ViewBuilder
    var timerOverlay: some View {
        if state == .connected, let connectedDate {
            TimelineView(.periodic(from: connectedDate, by: 1.0)) { context in
                Text(verbatim: fastHMS(from: connectedDate, to: context.date))
                    .font(.system(size: Constants.labelFontSize))
                    .foregroundColor(Color.Nym.primary)
                    .monospacedDigit()
            }
            .id(connectedDate)
            .offset(y: Constants.timerTopOffset)
        }
    }

    var sweepAnimation: Animation {
        .timingCurve(0.4, 0, 0.2, 1, duration: mode == .fast ? 0.8 : 1.2)
    }

    /// The connection marker state (#7063) this arc phase draws.
    var markerState: MarkerState {
        switch state {
        case .disconnected, .offline:
            return .notProtected
        case .step(let step):
            return .connecting(step: step.markerStep)
        case .awaitingGatewayConsent:
            return .connecting(step: (lastStep ?? .authenticatingAccount).markerStep)
        case .connected:
            return .connected
        case .failed:
            return .error
        case .blocked:
            return .blocked
        case .canceling:
            return .stopping
        }
    }

    /// Setup yellow while connecting, error red on error, brand green otherwise.
    var fillColor: Color {
        switch markerState {
        case .error:
            return Color.Nym.error
        case .connecting:
            return colorScheme == .dark ? Constants.setupDark : Constants.setupLight
        case .notProtected, .connected, .blocked, .stopping:
            return Constants.fastFill
        }
    }

    var trackColor: Color {
        colorScheme == .dark ? Constants.track : Color.black.opacity(0.12)
    }

    var fillOpacity: Double {
        switch state {
        case .canceling:
            return 0.15
        case .failed:
            return Constants.errorFillOpacity
        default:
            return 1.0
        }
    }

    var sphereOpacity: Double {
        switch state {
        case .disconnected, .offline:
            return 0.85
        case .step, .awaitingGatewayConsent:
            return 1.0
        case .connected, .blocked:
            return 1.0
        case .failed:
            return 1.0
        case .canceling:
            return 0.7
        }
    }

    var haloColor: Color {
        guard state == .connected else { return .clear }
        return Color.Nym.primary.opacity(0.55)
    }

    var haloRadius: CGFloat {
        mode == .fast ? 26 : 22
    }

    var labelText: String {
        switch state {
        case .disconnected, .offline:
            return "arcProgress.notProtected".localizedString
        case .blocked:
            return "arcProgress.blocked".localizedString
        case .connected:
            switch mode {
            case .fast:
                return "arcProgress.fastModeProtection".localizedString
            case .anonymous:
                return "arcProgress.anonymousModeProtection".localizedString
            }
        case .failed:
            return "arcProgress.connectionFailed".localizedString
        case .canceling:
            return "arcProgress.disconnecting".localizedString
        case .awaitingGatewayConsent:
            return "gatewayIndependence.warning.title".localizedString
        case .step(let step):
            return stepLabel(step)
        }
    }

    /// Caption under the label: why the device is offline (blocked or not
    /// protected), else the gateway-independence warning while connecting.
    var secondaryCaptionKey: String? {
        switch state {
        case .blocked:
            return "home.deviceNoInternetReconnect"
        case .offline:
            return "home.deviceNoInternet"
        default:
            return showsSecondaryWarning ? "gatewayIndependence.warning.title" : nil
        }
    }

    var showsSecondaryWarning: Bool {
        guard showsIndependenceWarning else {
            return false
        }
        // Only while connecting: the connected state owns the timer overlay
        // that sits where this caption would render.
        if case .step = state {
            return true
        }
        return false
    }

    var labelColor: Color {
        switch state {
        case .failed:
            return Constants.errorFill
        case .connected, .step, .awaitingGatewayConsent:
            return Color.Nym.primary
        case .blocked:
            return Color.Nym.textSecondary
        case .disconnected, .offline, .canceling:
            return Color.Nym.textTertiary
        }
    }

    func progress(for ring: Ring) -> CGFloat {
        switch state {
        case .connected:
            return 1.0
        case .disconnected, .offline:
            return 0.0
        case .failed:
            return 1.0
        case .blocked:
            // only the outer ring, which blinks
            return ring == .outer ? 1.0 : 0.0
        case .awaitingGatewayConsent:
            // Hold whatever ring the connect had already reached (macOS loads
            // up to the middle ring before the error); fall back to the outer
            // ring when nothing was reached yet (iOS pre-flight). Never unload.
            return progressForStep(lastStep ?? .authenticatingAccount, ring: ring)
        case .step(let step):
            return progressForStep(step, ring: ring)
        case .canceling:
            return progressForStep(lastStep, ring: ring)
        }
    }

    func progressForStep(_ step: ArcProgressState.Step?, ring: Ring) -> CGFloat {
        guard let step else {
            return 0.0 }
        let activeRing = step.ring
        if ring.index < activeRing.index {
            return 1.0 }
        if ring.index > activeRing.index {
            return 0.0 }
        return step.isFirstHalf ? 0.5 : 1.0
    }

    func recordStepIfActive(_ state: ArcProgressState) {
        if case .step(let step) = state {
            lastStep = step
        }
    }

    func stepLabel(_ step: ArcProgressState.Step) -> String {
        switch step {
        case .initializingNym:
            return "arcProgress.step.initializingNym".localizedString
        case .authenticatingAccount:
            return "arcProgress.step.authenticatingAccount".localizedString
        case .downloadingZkNyms:
            return "arcProgress.step.downloadingZkNyms".localizedString
        case .updatingServerList:
            return "arcProgress.step.updatingServerList".localizedString
        case .choosingBestServers:
            return "arcProgress.step.choosingBestServers".localizedString
        case .registeringWithServers:
            return "arcProgress.step.registeringWithServers".localizedString
        case .establishingConnection:
            return "arcProgress.step.establishingConnection".localizedString
        }
    }

    /// On error every ring and its track loses a segment centred on
    /// 3 o'clock, so the state reads without colour. Rings are rotated so the
    /// path starts at 12 o'clock; the dash phase puts the gap at 0.2 to 0.3.
    func ringStroke(for ring: Ring) -> StrokeStyle {
        guard markerState == .error else {
            return StrokeStyle(lineWidth: Constants.strokeWidth, lineCap: .round)
        }
        let circumference = CGFloat.pi * ring.diameter
        return StrokeStyle(
            lineWidth: Constants.strokeWidth,
            lineCap: .round,
            dash: [circumference * (1 - Constants.errorGap), circumference * Constants.errorGap],
            dashPhase: circumference * (1 - Constants.errorGapStart)
        )
    }

    /// Uniform scale applied to the whole arc+label+timer block so it
    /// fits within `availableHeight` when the drawer eats most of the
    /// screen. The arc is `.position`-centered in the parent, so timer
    /// (sitting below the VStack) sets the binding constraint:
    /// twice the below-VStack extent + the VStack height must clear the
    /// available height.
    var contentScale: CGFloat {
        guard let availableHeight, availableHeight > 0 else { return 1 }
        let required = Constants.requiredCenteredHeight
        guard required > 0 else { return 1 }
        return min(1, max(Constants.minContentScale, availableHeight / required))
    }

    func fastHMS(from start: Date, to now: Date) -> String {
        let total = max(0, Int(now.timeIntervalSince(start)))
        let hours = total / 3600
        let minutes = (total / 60) % 60
        let seconds = total % 60
        return String(format: "%02d:%02d:%02d", hours, minutes, seconds)
    }

    enum Ring {
        case outer, middle, inner

        var diameter: CGFloat {
            switch self {
            case .outer:
                return 92.4 * 2
            case .middle:
                return 78.4 * 2
            case .inner:
                return 64.4 * 2
            }
        }

        var index: Int {
            switch self {
            case .outer:
                return 0
            case .middle:
                return 1
            case .inner:
                return 2
            }
        }
    }

    enum Constants {
        static let strokeWidth: CGFloat = 6
        static let canvasSize: CGFloat = 92.4 * 2 + strokeWidth
        static let sphereDiameter: CGFloat = 64.4 * 2 * 0.56
        static let glowDiameter: CGFloat = 64.4 * 2 * 0.9

        static let labelFontSize: CGFloat = 11
        static let warningFontSize: CGFloat = 10
        static let warningTopOffset: CGFloat = 18
        static let labelTopSpacing: CGFloat = 14
        static let labelMinHeight: CGFloat = 14
        /// Vertical offset from ArcView's top edge to the timer baseline.
        /// Places the timer ~4pt below the bottom of the label line so it
        /// hangs visually below ArcView's frame without contributing to
        /// parent layout (overlay isn't clipped by default).
        static let timerTopOffset: CGFloat = canvasSize + labelTopSpacing + labelMinHeight + 4

        /// Height required so the arc's visual bounds (VStack + timer
        /// overlay) fit when `.position`-centered in the parent. Timer
        /// extends past the VStack bottom by `timerTopOffset + timer
        /// height - vstackHeight`; doubling that overhang and adding
        /// the VStack itself yields the minimum centered envelope.
        static var requiredCenteredHeight: CGFloat {
            let vstackHeight = canvasSize + labelTopSpacing + labelMinHeight
            let timerOverhang = max(0, timerTopOffset + labelMinHeight - vstackHeight)
            let safety: CGFloat = 36
            return vstackHeight + 2 * timerOverhang + 2 * safety
        }

        /// Floor on `scaleEffect` so the arc remains recognizable when
        /// available room is tiny. Below this we accept a small overlap
        /// behind the opaque drawer rather than render an illegible mark.
        static let minContentScale: CGFloat = 0.4

        static let glowConnectedOpacity: Double = 0.55

        static let fastFill      = Color.Nym.primary
        static let track         = Color.white.opacity(0.15)
        static let errorFill     = Color.Nym.error.opacity(0.60)
        static let errorFillOpacity: Double = 0.85
        /// Error gap: 10% of the circle, centred on 3 o'clock.
        static let errorGap: CGFloat = 0.1
        static let errorGapStart: CGFloat = 0.25 + errorGap / 2
        /// Setup yellow for the connecting fill.
        static let setupDark  = Color(red: 0xF4 / 255, green: 0xB9 / 255, blue: 0x40 / 255)
        static let setupLight = Color(red: 0x9A / 255, green: 0x5B / 255, blue: 0x00 / 255)
        static let errorTint     = Color.Nym.error.opacity(0.08)
    }
}

private extension ArcProgressState.Step {
    var ring: ArcProgressView.Ring {
        switch self {
        case .initializingNym, .authenticatingAccount, .downloadingZkNyms:
            return .outer
        case .updatingServerList, .choosingBestServers:
            return .middle
        case .registeringWithServers, .establishingConnection:
            return .inner
        }
    }

    var isFirstHalf: Bool {
        switch self {
        case .initializingNym, .updatingServerList, .registeringWithServers:
            return true
        case .authenticatingAccount, .downloadingZkNyms, .choosingBestServers, .establishingConnection:
            return false
        }
    }
}
