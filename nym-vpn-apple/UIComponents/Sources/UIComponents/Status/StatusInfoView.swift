import SwiftUI
import Theme

public struct StatusInfoView: View {
    @Environment(\.scenePhase) private var scenePhase
    @Binding private var connectedDate: Date?
    @Binding private var infoState: StatusInfoState

    public init(
        connectedDate: Binding<Date?>,
        infoState: Binding<StatusInfoState>
    ) {
        _connectedDate = connectedDate
        _infoState = infoState
    }

    public var body: some View {
        VStack(spacing: 8) {
            infoLabel()
                .onTapGesture {
                    if case let .error(message) = infoState {
                        copyToPasteboard(text: message)
                    }
                }
            timeConnectedLabel()
        }
    }
}

private extension StatusInfoView {
    @ViewBuilder
    func infoLabel() -> some View {
        Text(infoState.localizedTitle)
            .foregroundStyle(infoState.textColor)
            .textStyle(.Body.Medium.regular)
            .lineLimit(3, reservesSpace: infoState.localizedTitle.count > 30)
            .multilineTextAlignment(.center)
    }

    @ViewBuilder
    func timeConnectedLabel() -> some View {
        // Show timer only when we have internet AND have a start date
        let shouldShowTimer = !(infoState == .noInternet || infoState == .noInternetReconnect)

        if scenePhase == .active, shouldShowTimer, let start = connectedDate {
            TimelineView(.periodic(from: start, by: 1.0)) { context in
                let timeElapsed = fastHMS(from: start, to: context.date)
                Text(verbatim: timeElapsed)
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Body.Medium.regular)
                    .monospacedDigit()
            }
            .id(connectedDate)
        } else {
            Text(" ").frame(maxWidth: .infinity)
        }
    }
}

private extension StatusInfoView {
    static let digits: [String] = (0...59).map { $0 < 10 ? "0\($0)" : "\($0)" }

    func fastHMS(from start: Date, to now: Date) -> String {
        let total = max(0, Int(now.timeIntervalSince(start)))
        let hours = total / 3600
        let minutes = (total / 60) % 60
        let seconds = total % 60

        let hh = hours < 10 ? "0\(hours)" : "\(hours)"
        return "\(hh):\(Self.digits[minutes]):\(Self.digits[seconds])"
    }

    func copyToPasteboard(text: String) {
#if os(iOS)
        UIPasteboard.general.string = text
#elseif os(macOS)
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(text, forType: .string)
#endif
    }
}
