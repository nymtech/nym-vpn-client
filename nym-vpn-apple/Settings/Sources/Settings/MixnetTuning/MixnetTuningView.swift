import SwiftUI
import AppSettings
import Constants
import ConnectionManager
import ConnectionTypes
import ExternalLinkManager
import ImpactGenerator
import MessageModels
import Theme
import UIComponents

struct MixnetTuningView: View {
    private let continuousTrafficOptions = ContinuousTraffic.allCases
    private let coverTrafficOptions = BackgroundTraffic.allCases
    private let mixingDelayValues = Array(0...200)

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
    @State private var isSaveChangesSnackbarDisplayed = false

    @State private var config: MixnetTuningConfig?

    var body: some View {
        BaseView(
            pageTitleKey: "settings.mixnetTuning.title",
            leftNavButton: CustomNavBarButton(type: .back) { navigateBack() },
            content: {
                content
            }
        )
        .snackbar(
            isDisplayed: $isSaveChangesSnackbarDisplayed,
            message: SnackBarMessage(text: "mixnetTuning.snackbar.saved".localizedString, style: .info)
        )
        .task {
            let oldConfig = connectionManager.connectionConfig.mixnetTuningConfig
            config = oldConfig
            coverTrafficIndex = Double(oldConfig.backgroundTraffic.rawValue)
            continuousTrafficIndex = Double(oldConfig.continuousTraffic.rawValue)
            mixingDelayIndex = Double(oldConfig.averagePacketDelay)
            isSendTrafficContinuouslyOn = !oldConfig.disablePoissonRate

            updateLatencyRTT()
            updateIsSaveButtonEnabled()
        }

        .onChange(of: coverTrafficIndex) { _, newValue in
            config?.backgroundTraffic = BackgroundTraffic.fromIndex(newValue)
            updateIsSaveButtonEnabled()
        }
        .onChange(of: continuousTrafficIndex) { _, newValue in
            config?.continuousTraffic = ContinuousTraffic.fromIndex(newValue)
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
                .nymText(color: NymColor.gray1, style: .Body.Medium.regular)
            Spacer()
        }
    }

    var continuousTrafficMbps: ContinuousTraffic {
        continuousTrafficOptions[Int(continuousTrafficIndex)]
    }

    var performanceSection: some View {
        ElevationSectionView {
            Text("mixnetTunning.expectedPerformance.title".localizedString)
                .nymText(color: NymColor.gray1, style: .Body.Medium.regular)
            Spacer()
                .frame(height: 12)
            performanceCell(
                title: "mixnetTuning.speed".localizedString,
                subtitle: "\("mixnetTuning.upTo".localizedString) \(continuousTrafficMbps.uiValue) Mbps"
            )
            separatorLine()
            performanceCell(
                title: "mixnetTuning.latency".localizedString,
                subtitle: "\("mixnetTuning.atLeast".localizedString) \(latency) ms"
            )
            Spacer()
                .frame(height: 12)
            Text("mixnetTuning.expectedRTT".localizedString)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Small.regular)
        }
    }

    func performanceCell(title: String, subtitle: String) -> some View {
        HStack(spacing: 0) {
            Text(title)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
            Spacer()
            Text(subtitle)
                .foregroundStyle(NymColor.action)
                .textStyle(.Body.Medium.regular)
        }
    }

    func separatorLine() -> some View {
        Rectangle()
            .foregroundColor(NymColor.gray2)
            .frame(height: 1)
            .padding(.vertical, 12)
    }

    var trafficSection: some View {
        SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    viewModel: ToggleViewModel(
                        isOn: $isSendTrafficContinuouslyOn,
                        action: { isOn in
                            config?.disablePoissonRate = !isOn
                            updateIsSaveButtonEnabled()
                        }
                    )
                ),
                title: "mixnetTuning.sendTrafficContinously".localizedString,
                position: .init(isFirst: true, isLast: true),
                action: {}
            ),
            customContent: {
                trafficSubsection
            }
        )
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
            .nymText(color: NymColor.warning, style: .Body.Small.regular)
        Spacer()
            .frame(height: 16)
        Text("mixnetTuning.backgroundCoverTrafficState.title".localizedString)
            .nymText(color: NymColor.primary, style: .Headline.Small.regular)
        Spacer()
            .frame(height: 16)
        Text("mixnetTuning.backgroundCoverTrafficState.subtitle".localizedString)
            .nymText(color: NymColor.gray1, style: .Body.Small.regular)
        Spacer()
            .frame(height: 16)
        coverTrafficSliderSection
        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder var coverTrafficSliderSection: some View {
        HStack(spacing: 0) {
            Text("mixnetTuning.lessBatteryData".localizedString)
                .nymText(color: NymColor.gray1, style: .Body.Small.regular)
            Spacer()
            Text("mixnetTuning.maximumAnonimity".localizedString)
                .nymText(color: NymColor.gray1, style: .Body.Small.regular)
        }
        Spacer()
            .frame(height: 16)
        Slider(value: $coverTrafficIndex, in: 0.0...Double(coverTrafficOptions.count - 1), step: 1)
            .tint(NymColor.accent)
        Spacer()
            .frame(height: 16)
        HStack(spacing: 0) {
            Text("\("mixnetTuning.base".localizedString)\n")
                .nymText(color: NymColor.primary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
            Spacer()
            Text("\("mixnetTuning.balanced".localizedString)\n5x")
                .nymText(color: NymColor.primary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
            Spacer()
            Text("\("mixnetTuning.medium".localizedString)\n10x")
                .nymText(color: NymColor.primary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
            Spacer()
            Text("\("mixnetTuning.high".localizedString)\n20x")
                .nymText(color: NymColor.primary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
        }
    }

    @ViewBuilder var continiuosTrafficSection: some View {
        Text("mixnetTuning.sendTraffic.on".localizedString)
            .nymText(color: NymColor.gray1, style: .Body.Small.regular)
        Spacer()
            .frame(height: 16)
        HStack(spacing: 0) {
            Text("mixnetTuning.lessBatteryData".localizedString)
                .nymText(color: NymColor.gray1, style: .Body.Small.regular)
            Spacer()
            Text("mixnetTuning.maximumAnonimity".localizedString)
                .nymText(color: NymColor.gray1, style: .Body.Small.regular)
        }
        Spacer()
            .frame(height: 16)
        Slider(value: $continuousTrafficIndex, in: 0.0...Double(continuousTrafficOptions.count - 1), step: 1)
            .tint(NymColor.accent)
        Spacer()
            .frame(height: 16)
        HStack(spacing: 0) {
            Text("\("mixnetTuning.low".localizedString)\n0.7 Mbps")
                .nymText(color: NymColor.primary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
            Spacer()
            Text("\("mixnetTuning.balanced".localizedString)\n1 Mbps")
                .nymText(color: NymColor.primary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
            Spacer()
            Text("\("mixnetTuning.high".localizedString)\n2 Mbps")
                .nymText(color: NymColor.primary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
        }
        Spacer()
            .frame(height: 16)
    }

    var delaySection: some View {
        ElevationSectionView {
            Text("mixnetTuning.mixingDelays".localizedString)
                .nymText(color: NymColor.primary, style: .Headline.Small.regular)
            Spacer()
                .frame(height: 16)
            if mixingDelayIndex == 0 {
                Text("⚠️ \("mixnetTuning.mixingDelay.off".localizedString)")
                    .nymText(color: NymColor.warning, style: .Body.Small.regular)
            } else {
                Text("mixnetTuning.mixingDelay.on".localizedString)
                    .nymText(color: NymColor.gray1, style: .Body.Small.regular)
            }
            Spacer()
                .frame(height: 16)
            sliderValue
            sliderExplanation
        }
    }

    @ViewBuilder var sliderValue: some View {
        HStack(spacing: 0) {
            Text("mixnetTuning.fasterSpeed".localizedString)
                .nymText(color: NymColor.gray1, style: .Body.Small.regular)
            Spacer()
            Text("mixnetTuning.maximumAnonimity".localizedString)
                .nymText(color: NymColor.gray1, style: .Body.Small.regular)
        }
        Spacer()
            .frame(height: 16)
        Slider(value: $mixingDelayIndex, in: 0.0...Double(mixingDelayValues.count - 1), step: 1)
            .tint(NymColor.accent)
            .onChange(of: mixingDelayIndex) { _, _ in
                updateLatencyRTT()
            }
        Spacer()
            .frame(height: 16)
    }

    var sliderExplanation: some View {
        HStack(spacing: 0) {
            Text("\("mixnetTuning.low".localizedString)\n0 ms")
                .nymText(color: NymColor.primary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
            Spacer()
            if mixingDelayIndex != 15 {
                Text("\("mixnetTuning.current".localizedString)\n \(Int(mixingDelayIndex)) ms")
                    .nymText(color: NymColor.info, style: .Body.Medium.regular)
                    .multilineTextAlignment(.center)
            } else {
                Text("\("mixnetTuning.default".localizedString)\n15 ms")
                    .nymText(color: NymColor.info, style: .Body.Medium.regular)
                    .multilineTextAlignment(.center)
            }
            Spacer()
            Text("\("mixnetTuning.high".localizedString)\n200 ms")
                .nymText(color: NymColor.primary, style: .Body.Medium.regular)
                .multilineTextAlignment(.center)
        }
    }

    var learnMoreAttributtedString: AttributedString {
        var first = AttributedString("mixnetTuning.learnMore".localizedString)
        first.underlineStyle = .single
        first.foregroundColor = NymColor.primary
        first.link = URL(string: Constants.mixnetParametersLearnMoreURL.rawValue)
        return first
    }

    var learnMoreLink: some View {
        ExternalLink(text: learnMoreAttributtedString, color: NymColor.primary, style: .Body.Small.regular)
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
        GenericButton(title: "mixnetTuning.reset".localizedString, style: .primaryBorderOnly, height: 36)
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
        let connectionConfig = connectionManager.connectionConfig
        connectionConfig.mixnetTuningConfig = config
        connectionManager.connectionConfig = connectionConfig
        updateIsSaveButtonEnabled()

        withAnimation {
            isSaveChangesSnackbarDisplayed = true
            Task { @MainActor in
                try? await Task.sleep(for: .seconds(3))
                isSaveChangesSnackbarDisplayed = false
            }
        }
    }

    func resetToDefaults() {
        mixingDelayIndex = 25
        continuousTrafficIndex = 1
        coverTrafficIndex = 0
        isSendTrafficContinuouslyOn = true
    }
}
