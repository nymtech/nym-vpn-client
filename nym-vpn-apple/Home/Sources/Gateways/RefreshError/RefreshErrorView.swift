import SwiftUI
import Combine
import Constants
import ExternalLinkManager
import UIComponents
import Theme

struct RefreshErrorView: View {
    @Binding private var isDisplayed: Bool
    private let refresh: (() -> Void)
    private let refreshCooldownDuration: TimeInterval

    @State private var remainingSeconds: Int = 0
    @State private var timer: AnyCancellable?

    static let refreshCooldownIsActiveKey = "refreshCooldownIsActive"
    @AppStorage(refreshCooldownIsActiveKey)
    private var refreshCooldownIsActive: Bool = false
    static let refreshCooldownEndTimeKey = "refreshCooldownEndTime"
    @AppStorage(refreshCooldownEndTimeKey)
    private var refreshCooldownEndTime: Double = 0

    init(isDisplayed: Binding<Bool>, refresh: @escaping () -> Void, refreshCooldownDuration: TimeInterval = 60.0) {
        _isDisplayed = isDisplayed
        self.refresh = refresh
        self.refreshCooldownDuration = refreshCooldownDuration
    }

    var body: some View {
        ZStack {
            Rectangle()
                .foregroundColor(.black)
                .opacity(0.3)
                .background(Color.clear)
                .contentShape(Rectangle())

            HStack {
                Spacer()
                    .frame(width: 40)

                VStack {
                    icon()
                    title()
                    Spacer()
                        .frame(height: 16)
                    subtitle()
                    okButton()
                    countdownLabel()
                }
                .padding(.horizontal, 24)
                .background(NymColor.elevation)
                .cornerRadius(16)

                Spacer()
                    .frame(width: 40)
            }
        }
        .edgesIgnoringSafeArea(.all)
        .onAppear {
            restoreOrStartTimer()
        }
    }
}

private extension RefreshErrorView {
    @ViewBuilder
    func icon() -> some View {
        Spacer()
            .frame(height: 24)

        GenericImage(imageName: "errorReporting")
            .frame(width: 24, height: 24)
            .foregroundStyle(NymColor.error)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func title() -> some View {
        Text("gatewaysView.serverListRefreshFailed.modal.title".localizedString)
            .textStyle(.Headline.Medium.regular)
            .foregroundStyle(NymColor.primary)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func subtitle() -> some View {
        Text("gatewaysView.serverListRefreshFailed.modal.subtitle".localizedString)
            .textStyle(.Body.Medium.regular)
            .foregroundStyle(NymColor.gray1)
            .multilineTextAlignment(.center)
    }

    @ViewBuilder
    func okButton() -> some View {
        GenericButton(title: "ok".localizedString, style: .primaryBorderOnly)
            .padding(.vertical, 24)
            .onTapGesture {
                isDisplayed.toggle()
            }
    }

    @ViewBuilder
    func countdownLabel() -> some View {
        VStack {
            if remainingSeconds > 0 {
                Text("\("gatewaysView.serverListRefreshFailed.modal.refreshInSeconds".localizedString) \(remainingSeconds) \("gatewaysView.serverListRefreshFailed.modal.seconds".localizedString)")
                    .textStyle(.Body.Medium.bold)
                    .foregroundStyle(NymColor.gray1)
            } else {
                GenericButton(
                    title: "gatewaysView.serverListRefreshFailed.modal.refreshServerListButton".localizedString,
                    style: .textOnly
                )
                .onTapGesture {
                    refresh()
                    isDisplayed.toggle()
                }
            }
        }
        .padding(.bottom, 24)
    }
}

private extension RefreshErrorView {
    var endTime: Date? {
        get {
            refreshCooldownEndTime > 0 ? Date(timeIntervalSince1970: refreshCooldownEndTime) : nil
        }
        nonmutating set {
            refreshCooldownEndTime = newValue?.timeIntervalSince1970 ?? 0
        }
    }

    func restoreOrStartTimer() {
        if refreshCooldownIsActive, let endTime {
            let now = Date()
            if now < endTime {
                refreshCooldownIsActive = true
                startTimer()
            } else {
                cleanup()
            }
        } else {
            start()
        }
    }

    func start() {
        let newEndTime = Date().addingTimeInterval(refreshCooldownDuration)
        endTime = newEndTime
        refreshCooldownIsActive = true
        startTimer()
    }

    func startTimer() {
        timer?.cancel()
        updateRemainingTime()

        timer = Timer.publish(every: 0.1, on: .main, in: .common)
            .autoconnect()
            .sink { _ in
                updateRemainingTime()
            }
    }

    func updateRemainingTime() {
        guard let endTime = endTime else {
            cleanup()
            return
        }

        let now = Date()
        let remaining = endTime.timeIntervalSince(now)

        if remaining > 0 {
            remainingSeconds = Int(ceil(remaining))
        } else {
            remainingSeconds = 0
            cleanup()
        }
    }

    func cleanup() {
        timer?.cancel()
        timer = nil
        endTime = nil
        refreshCooldownIsActive = false
    }
}
