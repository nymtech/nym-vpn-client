import SwiftUI
import Theme

public struct StatusInfoView: View {
    private let isSmallScreen: Bool

    @Binding private var timeConnected: Date?
    @Binding private var infoState: StatusInfoState

    public init(
        timeConnected: Binding<Date?>,
        infoState: Binding<StatusInfoState>,
        isSmallScreen: Bool
    ) {
        _timeConnected = timeConnected
        _infoState = infoState
        self.isSmallScreen = isSmallScreen
    }

    public var body: some View {
        infoLabel()
            .onTapGesture {
                switch infoState {
                case let .error(message):
                    copyToPasteboard(text: message)
                default:
                    break
                }
            }
        timeConnectedLabel()
    }
}

private extension StatusInfoView {
    @ViewBuilder
    func infoLabel() -> some View {
        Text(infoState.localizedTitle)
            .foregroundStyle(infoState.textColor)
            .textStyle(isSmallScreen ? .Label.Medium.primary : .Label.Large.bold)
            .lineLimit(3, reservesSpace: infoState.localizedTitle.count > 30 ? true : false)
            .multilineTextAlignment(.center)
            .transition(.opacity)
            .animation(.easeInOut, value: infoState.localizedTitle)
        Spacer()
            .frame(height: 8)
    }

    @ViewBuilder
    func timeConnectedLabel() -> some View {
        if infoState != .noInternet || infoState != .noInternetReconnect, let timeConnected {
            TimelineView(.periodic(from: timeConnected, by: 1.0)) { context in
                let timeElapsed = context.date.timeIntervalSince(timeConnected)

                let hours = Int(timeElapsed) / 3600
                let minutes = (Int(timeElapsed) % 3600) / 60
                let seconds = Int(timeElapsed) % 60

                Text("\(String(format: "%02d:%02d:%02d", hours, minutes, seconds))")
                    .foregroundStyle(NymColor.statusTimer)
                    .textStyle(isSmallScreen ? .Label.Medium.primary : .Label.Large.bold)
                    .transition(.opacity)
                    .animation(.easeInOut, value: timeConnected)
            }
        } else {
            Text(" ")
        }
    }
}

private extension StatusInfoView {
    func copyToPasteboard(text: String) {
#if os(iOS)
        UIPasteboard.general.string = text
#elseif os(macOS)
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(text, forType: .string)
#endif
    }
}
