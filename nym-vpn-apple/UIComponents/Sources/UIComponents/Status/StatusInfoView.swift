import SwiftUI
import AppSettings
import Theme

public struct StatusInfoView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @Binding private var timeConnectedString: String?
    @Binding private var infoState: StatusInfoState

    public init(
        timeConnectedString: Binding<String?>,
        infoState: Binding<StatusInfoState>
    ) {
        _timeConnectedString = timeConnectedString
        _infoState = infoState
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
            .textStyle(.Headline.Medium.regular)
            .lineLimit(3, reservesSpace: infoState.localizedTitle.count > 30 ? true : false)
            .multilineTextAlignment(.center)
            .transition(.opacity)
            .animation(.easeInOut, value: infoState.localizedTitle)
        Spacer()
            .frame(height: 8)
    }

    @ViewBuilder
    func timeConnectedLabel() -> some View {
        if infoState != .noInternet || infoState != .noInternetReconnect, let timeConnectedString {
            TimelineView(.animation(minimumInterval: 1.0, paused: false)) { _ in
                Text(timeConnectedString)
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Headline.Medium.regular)
                    .transition(.opacity)
                    .animation(.easeInOut, value: timeConnectedString)
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
