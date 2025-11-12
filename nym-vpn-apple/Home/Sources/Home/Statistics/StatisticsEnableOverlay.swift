import SwiftUI
import AppSettings
import UIComponents
import Theme

struct StatisticsEnableOverlay: View {
    @EnvironmentObject private var appSettings: AppSettings
    @Binding var isPresented: Bool

    var body: some View {
        ZStack {
            Rectangle()
                .foregroundColor(.black)
                .opacity(0.3)
                .background(.clear)
                .contentShape(Rectangle())

            HStack(spacing: 0) {
                VStack(alignment: .center, spacing: 0) {
                    icon
                    Spacer()
                        .frame(height: 16)
                    upgradeExperienceTitle
                    Spacer()
                        .frame(height: 16)
                    helpImproveSubtitle
                    Spacer()
                        .frame(height: 16)
                    yourDataSubtitle
                    Spacer()
                        .frame(height: 24)
                    enableNetworkStatsButton
                    Spacer()
                        .frame(height: 8)
                    notNowButton
                }
                .padding(24)
                .background(NymColor.elevation)
                .cornerRadius(16)
            }
            .frame(maxWidth: MagicNumbers.maxWidth)
            .padding(24)
        }
        .edgesIgnoringSafeArea(.all)
    }
}

private extension StatisticsEnableOverlay {
    var icon: some View {
        Image(systemName: "info.circle")
            .frame(width: 24, height: 24)
    }

    var upgradeExperienceTitle: some View {
        Text("statisticsOverlay.upgradeExperience".localizedString)
            .textStyle(.Headline.Medium.regular)
            .foregroundStyle(NymColor.primary)
            .multilineTextAlignment(.center)
    }

    var helpImproveSubtitle: some View {
        Text("statisticsOverlay.helpImprove".localizedString)
            .textStyle(.Body.Medium.regular)
            .foregroundStyle(NymColor.gray1)
            .multilineTextAlignment(.center)
    }

    var yourDataSubtitle: some View {
        Text("statisticsOverlay.yourData".localizedString)
            .textStyle(.Body.Medium.regular)
            .foregroundStyle(NymColor.gray1)
            .multilineTextAlignment(.center)
    }

    var enableNetworkStatsButton: some View {
        GenericButton(title: "statisticsOverlay.enableNetworkStats".localizedString)
            .onTapGesture {
                enableNetworkStatsDidTap()
            }
            .accessibilityAction {
                enableNetworkStatsDidTap()
            }
    }

    var notNowButton: some View {
        GenericButton(title: "statisticsOverlay.notNow".localizedString, style: .textOnly)
            .onTapGesture {
                dismiss()
            }
            .accessibilityAction {
                dismiss()
            }
    }
}

// MARK: - Actions -
private extension StatisticsEnableOverlay {
    func enableNetworkStatsDidTap() {
        appSettings.isStatisticsEnabled = true
        isPresented = false
    }

    func dismiss() {
        isPresented = false
    }
}
