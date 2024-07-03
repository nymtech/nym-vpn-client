import SwiftUI
import Theme

public struct StatusInfoView: View {
    private let isSmallScreen: Bool

    @Binding private var timeConnected: String
    @Binding private var infoState: StatusInfoState

    public init(
        timeConnected: Binding<String>,
        infoState: Binding<StatusInfoState>,
        isSmallScreen: Bool
    ) {
        _timeConnected = timeConnected
        _infoState = infoState
        self.isSmallScreen = isSmallScreen
    }

    public var body: some View {
        infoLabel()
        timeConnectedLabel(timeConnected: timeConnected)
    }
}

private extension StatusInfoView {
    @ViewBuilder
    func infoLabel() -> some View {
        Text(infoState.localizedTitle)
            .foregroundStyle(NymColor.statusInfoText)
            .textStyle(isSmallScreen ? .Label.Medium.primary : .Label.Large.bold)
        Spacer()
            .frame(height: 8)
    }

    @ViewBuilder
    func timeConnectedLabel(timeConnected: String) -> some View {
        Text("\(timeConnected)")
            .foregroundStyle(NymColor.statusTimer)
            .textStyle(isSmallScreen ? .Label.Medium.primary : .Label.Large.bold)
    }
}
