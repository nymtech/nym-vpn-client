import SwiftUI
import AppSettings
import Constants
import ConnectionManager
import ConnectionTypes
import ExternalLinkManager
import ImpactGenerator
import SnackbarManager
import Theme
import UIComponents
import NymVPNLib

struct MixnetTuningView: View {
    private let mixnetDefaults = MixnetTrafficDefaults()

    private var coverTrafficOptions: [BackgroundCoverTrafficRate] {
        mixnetDefaults.allBackgroundTraffic()
    }
    private var continuousTrafficOptions: [ContinuousTrafficSendingRate] {
        mixnetDefaults.allContinuousTraffic()
    }

    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var impactGenerator: ImpactGenerator
    @EnvironmentObject private var externalLinkManager: ExternalLinkManager
    @Binding private var path: NavigationPath
    @State private var latency = 700
    @State private var coverTrafficIndex: Double = 0
    @State private var continuousTrafficIndex: Double = 1
    @State private var mixingDelayIndex = 15.0
    @State private var isSendTrafficContinuouslyOn = false
    @State private var isSaveButtonDisabled = false

    @State private var config: MixnetTuningConfig?

    var body: some View {
        BaseView(
            pageTitleKey: "settings.mixnetTuning.title",
            leftNavButton: CustomNavBarButton(type: .back) { navigateBack() },
            content: {
                content
            }
        )
        .task {
            let oldConfig = connectionManager.connectionConfig.mixnetTuningConfig
            config = oldConfig
            coverTrafficIndex = Double(coverTrafficOptions.firstIndex(of: oldConfig.backgroundTraffic) ?? 0)
            continuousTrafficIndex = Double(continuousTrafficOptions.firstIndex(of: oldConfig.continuousTraffic) ?? 0)
            mixingDelayIndex = Double(oldConfig.averagePacketDelay)
            isSendTrafficContinuouslyOn = !oldConfig.disablePoissonRate

            updateLatencyRTT()
            updateIsSaveButtonEnabled()
        }

        .onChange(of: coverTrafficIndex) { _, newValue in
            config?.backgroundTraffic = coverTrafficOptions[Int(newValue)]
            updateIsSaveButtonEnabled()
        }
        .onChange(of: continuousTrafficIndex) { _, newValue in
            config?.continuousTraffic = continuousTrafficOptions[Int(newValue)]
            updateIsSaveButtonEnabled()
        }
        .onChange(of: mixingDelayIndex) { _, newValue in
            config?.averagePacketDelay = Int(newValue)
            updateIsSaveButtonEnabled()
        }
    }

    init(path: Binding<NavigationPath>) {
        _path = path
    }
}

// MARK: - Views -
private extension MixnetTuningView {
    @ViewBuilder var content: some View {
        Spacer()
            .frame(height: 24)
        subtitle
        Spacer()
            .frame(height: 24)
        performanceSection
        Spacer()
            .frame(height: 24)
        trafficSection
        Spacer()
            .frame(height: 24)
        delaySection
        Spacer()
            .frame(height: 24)
        learnMoreLink
        Spacer()
            .frame(height: 24)
        actionButtons
        Spacer()
            .frame(height: 24)
    }

    var subtitle: some View {
        HStack(spacing: 0) {
            Text("mixnetTunning.subtitle".localizedString)
                .nymText(color: Color.Nym.textSecondary, style: .Body.Medium.regular)
            Spacer()
        }
    }

    var continuousTrafficMbps: ContinuousTrafficSendingRate {
        continuousTrafficOptions[Int(continuousTrafficIndex)]
    }

    var performanceSection: some View {
        ElevationSectionView {
            Text("mixnetTunning.expectedPerformance.title".localizedString)
                .nymText(color: Color.Nym.textSecondary, style: .Body.Medium.regular)
            Spacer()
                .frame(height: 12)
            performanceCell(
                title: "mixnetTuning.packetRate".localizedString,
                subtitle: "\("mixnetTuning.upTo".localizedString) \(continuousTrafficMbps.uiThroughput) Mbps"
            )
            separatorLine()
            performanceCell(
                title: "mixnetTuning.latency".localizedString,
                subtitle: "\("mixnetTuning.atLeast".localizedString) \(latency) ms"
            )
            Spacer()
                .frame(height: 12)
            Text("mixnetTuning.expectedRTT".localizedString)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodySmall)
        }
    }

    func performanceCell(title: String, subtitle: String) -> some View {
        HStack(spacing: 0) {
            Text(title)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            Spacer()
            Text(subtitle)
                .foregroundStyle(Color.Nym.primary)
                .nymTextStyle(.bodyDefault)
        }
    }

    func separatorLine() -> some View {
        Rectangle()
            .foregroundColor(Color.Nym.textSecondary)
            .frame(height: 1)
            .padding(.vertical, 12)
    }

    var trafficSection: some View {
        SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    isOn: $isSendTrafficContinuouslyOn
                ),
                title: "mixnetTuning.sendTrafficContinously".localizedString,
                position: .init(isFirst: true, isLast: true),
                action: {}
            ),
            customContent: {
                trafficSubsection
            },
            combineAccessibilityChildren: false
        )
        .onChange(of: isSendTrafficContinuouslyOn) {
            config?.disablePoissonRate = !isSendTrafficContinuouslyOn
            updateIsSaveButtonEnabled()
        }
    }

    @ViewBuilder var trafficSubsection: some View {
        VStack(alignment: .leading, spacing: 0) {
            if !isSendTrafficContinuouslyOn {
                backgroundCoverTrafficSection
            } else {
                continiuosTrafficSection
            }
        }
        .padding(.horizontal, 16)
    }

    @ViewBuilder var backgroundCoverTrafficSection: some View {
        Text("⚠️ \("mixnetTuning.sendTraffic.off".localizedString)")
            .nymText(color: Color.Nym.warning, style: .Body.Small.regular)
        Spacer()
            .frame(height: 16)
        Text("mixnetTuning.backgroundCoverTrafficState.title".localizedString)
            .nymText(color: Color.Nym.textPrimary, style: .Headline.Small.regular)
        Spacer()
            .frame(height: 16)
        Text("mixnetTuning.backgroundCoverTrafficState.subtitle".localizedString)
            .nymText(color: Color.Nym.textSecondary, style: .Body.Small.regular)
        Spacer()
            .frame(height: 16)
        coverTrafficSliderSection
        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder var coverTrafficSliderSection: some View {
        HStack(spacing: 0) {
            Text("mixnetTuning.performance".localizedString)
                .nymText(color: Color.Nym.textSecondary, style: .Body.Small.regular)
            Spacer()
            Text("mixnetTuning.anonymity".localizedString)
                .nymText(color: Color.Nym.textSecondary, style: .Body.Small.regular)
        }
        Spacer()
            .frame(height: 16)
        Slider(value: snapping($coverTrafficIndex), in: 0.0...Double(coverTrafficOptions.count - 1))
            .tint(Color.Nym.primary)
            .accessibilityLabel("mixnetTuning.backgroundCoverTrafficState.title".localizedString)
            .accessibilityValue(coverTrafficAccessibilityValue)
            .accessibilityAdjustableAction { direction in
                adjustCoverTraffic(direction)
            }
        Spacer()
            .frame(height: 16)
        HStack(spacing: 0) {
            Text("\("mixnetTuning.base".localizedString)\n")
                .nymText(color: Color.Nym.textPrimary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
            Spacer()
            Text("\("mixnetTuning.balanced".localizedString)\n5x")
                .nymText(color: Color.Nym.textPrimary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
            Spacer()
            Text("\("mixnetTuning.medium".localizedString)\n10x")
                .nymText(color: Color.Nym.textPrimary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
            Spacer()
            Text("\("mixnetTuning.high".localizedString)\n20x")
                .nymText(color: Color.Nym.textPrimary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
        }
    }

    @ViewBuilder var continiuosTrafficSection: some View {
        Text("mixnetTuning.sendTraffic.on".localizedString)
            .nymText(color: Color.Nym.textSecondary, style: .Body.Small.regular)
        Spacer()
            .frame(height: 16)
        HStack(spacing: 0) {
            Text("mixnetTuning.performance".localizedString)
                .nymText(color: Color.Nym.textSecondary, style: .Body.Small.regular)
            Spacer()
            Text("mixnetTuning.anonymity".localizedString)
                .nymText(color: Color.Nym.textSecondary, style: .Body.Small.regular)
        }
        Spacer()
            .frame(height: 16)
        Slider(value: snapping($continuousTrafficIndex), in: 0.0...Double(continuousTrafficOptions.count - 1))
            .tint(Color.Nym.primary)
            .accessibilityLabel("mixnetTuning.sendTrafficContinously".localizedString)
            .accessibilityValue(continuousTrafficAccessibilityValue)
            .accessibilityAdjustableAction { direction in
                adjustContinuousTraffic(direction)
            }
        Spacer()
            .frame(height: 16)
        HStack(spacing: 0) {
            Text("\("mixnetTuning.low".localizedString)\n0.7 Mbps")
                .nymText(color: Color.Nym.textPrimary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
            Spacer()
            Text("\("mixnetTuning.balanced".localizedString)\n1 Mbps")
                .nymText(color: Color.Nym.textPrimary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
            Spacer()
            Text("\("mixnetTuning.high".localizedString)\n2 Mbps")
                .nymText(color: Color.Nym.textPrimary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
        }
        Spacer()
            .frame(height: 16)
    }

    var delaySection: some View {
        ElevationSectionView {
            Text("mixnetTuning.packetMixingProfile".localizedString)
                .nymText(color: Color.Nym.textPrimary, style: .Headline.Small.regular)
            Spacer()
                .frame(height: 16)
            if mixingDelayIndex == 0 {
                Text("⚠️ \("mixnetTuning.mixingDelay.off".localizedString)")
                    .nymText(color: Color.Nym.warning, style: .Body.Small.regular)
            } else {
                Text("mixnetTuning.mixingDelay.on".localizedString)
                    .nymText(color: Color.Nym.textSecondary, style: .Body.Small.regular)
            }
            Spacer()
                .frame(height: 16)
            sliderValue
            sliderExplanation
        }
    }

    @ViewBuilder var sliderValue: some View {
        HStack(spacing: 0) {
            Text("mixnetTuning.performance".localizedString)
                .nymText(color: Color.Nym.textSecondary, style: .Body.Small.regular)
            Spacer()
            Text("mixnetTuning.anonymity".localizedString)
                .nymText(color: Color.Nym.textSecondary, style: .Body.Small.regular)
        }
        Spacer()
            .frame(height: 16)
        Slider(
            value: snapping($mixingDelayIndex),
            in: Double(mixnetDefaults.defaultMixingDelay().minValue)...Double(mixnetDefaults.defaultMixingDelay().maxValue)
        )
            .tint(Color.Nym.primary)
            .accessibilityLabel("mixnetTuning.mixingDelays".localizedString)
            .accessibilityValue(mixingDelayAccessibilityValue)
            .accessibilityAdjustableAction { direction in
                adjustMixingDelay(direction)
            }
            .onChange(of: mixingDelayIndex) { _, _ in
                updateLatencyRTT()
            }
        Spacer()
            .frame(height: 16)
    }

    var sliderExplanation: some View {
        let defaultDelay = mixnetDefaults.defaultMixingDelay()
        return HStack(spacing: 0) {
            Text("\("mixnetTuning.low".localizedString)\n\(defaultDelay.minValue) ms")
                .nymText(color: Color.Nym.textPrimary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
            Spacer()
            if mixingDelayIndex != Double(defaultDelay.defaultValue) {
                Text("\("mixnetTuning.current".localizedString)\n \(Int(mixingDelayIndex)) ms")
                    .nymText(color: Color.Nym.info, style: .Body.Medium.regular)
                    .multilineTextAlignment(.center)
            } else {
                Text("\("mixnetTuning.default".localizedString)\n\(defaultDelay.defaultValue) ms")
                    .nymText(color: Color.Nym.info, style: .Body.Medium.regular)
                    .multilineTextAlignment(.center)
            }
            Spacer()
            Text("\("mixnetTuning.high".localizedString)\n\(defaultDelay.maxValue) ms")
                .nymText(color: Color.Nym.textPrimary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
        }
    }

    var learnMoreAttributtedString: AttributedString {
        var first = AttributedString("mixnetTuning.learnMore".localizedString)
        first.underlineStyle = .single
        first.foregroundColor = Color.Nym.textPrimary
        first.link = URL(string: Constants.mixnetParametersLearnMoreURL.rawValue)
        return first
    }

    var learnMoreLink: some View {
        ExternalLink(text: learnMoreAttributtedString, color: Color.Nym.textPrimary, style: .Body.Small.regular)
            .onTapGesture {
                try? externalLinkManager.openExternalURL(urlString: Constants.mixnetParametersLearnMoreURL.rawValue)
            }
    }

    @ViewBuilder var actionButtons: some View {
        GenericButton(
            title: "mixnetTuning.saveCustomSettings".localizedString,
            height: 42,
            isDisabled: $isSaveButtonDisabled
        )
        .onTapGesture {
            saveSettings()
        }
        .accessibilityAction {
            saveSettings()
        }
        Spacer()
            .frame(height: 24)
        GenericButton(title: "mixnetTuning.reset".localizedString, style: .primaryBorderOnly, height: 42)
            .onTapGesture {
                resetToDefaults()
            }
            .accessibilityAction {
                resetToDefaults()
            }
    }
}

// MARK: - Helpers -
private extension MixnetTuningView {
    /// Snaps a continuous slider value to whole steps without passing `step:` to `Slider`.
    /// Native `Slider(step:)` keeps an internal continuous gesture position that fights the
    /// snapped value at the boundaries; with few discrete values the gap is large, so the
    /// thumb visibly oscillates ("vibrates") at the extremes. Rounding in the binding and
    /// dropping `step:` removes that fight while keeping discrete snapping.
    func snapping(_ value: Binding<Double>) -> Binding<Double> {
        Binding(
            get: { value.wrappedValue },
            set: { newValue in
                let rounded = newValue.rounded()
                guard rounded != value.wrappedValue else { return }
                value.wrappedValue = rounded
            }
        )
    }

    var coverTrafficAccessibilityValue: String {
        switch safeCoverTrafficIndex {
        case 0:
            "mixnetTuning.base".localizedString
        case 1:
            "mixnetTuning.balanced".localizedString
        case 2:
            "mixnetTuning.medium".localizedString
        case 3:
            "mixnetTuning.high".localizedString
        default:
            "mixnetTuning.base".localizedString
        }
    }

    var continuousTrafficAccessibilityValue: String {
        "\(continuousTrafficOptions[safeContinuousTrafficIndex].uiThroughput) Mbps"
    }

    var mixingDelayAccessibilityValue: String {
        "\(safeMixingDelayValue) ms"
    }

    var safeCoverTrafficIndex: Int {
        min(max(Int(coverTrafficIndex.rounded()), 0), coverTrafficOptions.count - 1)
    }

    var safeContinuousTrafficIndex: Int {
        min(max(Int(continuousTrafficIndex.rounded()), 0), continuousTrafficOptions.count - 1)
    }

    var safeMixingDelayValue: Int {
        let defaultDelay = mixnetDefaults.defaultMixingDelay()
        return min(
            max(Int(mixingDelayIndex.rounded()), Int(defaultDelay.minValue)),
            Int(defaultDelay.maxValue)
        )
    }

    func adjustCoverTraffic(_ direction: AccessibilityAdjustmentDirection) {
        switch direction {
        case .increment:
            coverTrafficIndex = min(coverTrafficIndex + 1, Double(coverTrafficOptions.count - 1))
        case .decrement:
            coverTrafficIndex = max(coverTrafficIndex - 1, 0)
        @unknown default:
            break
        }
    }

    func adjustContinuousTraffic(_ direction: AccessibilityAdjustmentDirection) {
        switch direction {
        case .increment:
            continuousTrafficIndex = min(continuousTrafficIndex + 1, Double(continuousTrafficOptions.count - 1))
        case .decrement:
            continuousTrafficIndex = max(continuousTrafficIndex - 1, 0)
        @unknown default:
            break
        }
    }

    func adjustMixingDelay(_ direction: AccessibilityAdjustmentDirection) {
        let defaultDelay = mixnetDefaults.defaultMixingDelay()
        switch direction {
        case .increment:
            mixingDelayIndex = min(mixingDelayIndex + 1, Double(defaultDelay.maxValue))
        case .decrement:
            mixingDelayIndex = max(mixingDelayIndex - 1, Double(defaultDelay.minValue))
        @unknown default:
            break
        }
    }

    func updateLatencyRTT() {
        let latencyRaw = 2 * (6 * 50 + 3 * Int(mixingDelayIndex))
        latency = ((latencyRaw + 5) / 10) * 10
    }

    func updateIsSaveButtonEnabled() {
        isSaveButtonDisabled = connectionManager.connectionConfig.mixnetTuningConfig == config
    }
}

// MARK: - Actions -
private extension MixnetTuningView {
    func navigateBack() {
        guard !path.isEmpty else { return }
        impactGenerator.softImpact()
        path.removeLast()
    }

    func saveSettings() {
        guard let config else { return }
        connectionManager.setMixnetTuningConfig(config)
        updateIsSaveButtonEnabled()

        SnackbarManager.shared.enqueue(
            SnackbarItem(
                style: .confirmation,
                title: "mixnetTuning.snackbar.saved".localizedString
            )
        )
    }

    func resetToDefaults() {
        let defaultDelay = mixnetDefaults.defaultMixingDelay()
        mixingDelayIndex = Double(defaultDelay.defaultValue)

        let defaultBgTraffic = mixnetDefaults.defaultBackgroundTraffic()
        coverTrafficIndex = Double(coverTrafficOptions.firstIndex(of: defaultBgTraffic) ?? 0)

        let defaultContTraffic = mixnetDefaults.defaultContinuousTraffic()
        continuousTrafficIndex = Double(continuousTrafficOptions.firstIndex(of: defaultContTraffic) ?? 0)

        isSendTrafficContinuouslyOn = !mixnetDefaults.defaultDisablePoissionRate()
    }
}
