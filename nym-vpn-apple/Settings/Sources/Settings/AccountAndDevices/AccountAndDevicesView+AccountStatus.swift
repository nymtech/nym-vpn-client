import SwiftUI
import ConnectionTypes
import UIComponents
import Theme

// MARK: - Account Status Section -
extension AccountAndDevicesView {
    func accountStatusSection() -> some View {
        VStack(spacing: 0) {
            accountStatusHeader()
            if let accountSummary = credentialsManager.accountSummary, accountSummary.isActive {
                accountStatusBandwidth(accountSummary: accountSummary)
                Divider()
                    .frame(height: 1)
                    .overlay(NymColor.background)
                    .padding(.horizontal, 16)
                accountStatusResetDate(accountSummary: accountSummary)
                renewNowRow(accountSummary: accountSummary)
            } else {
                Divider()
                    .frame(height: 1)
                    .overlay(NymColor.background)
                accountStatusInactive()
            }
        }
        .background(NymColor.elevation)
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }

    func accountStatusInactive() -> some View {
        VStack(spacing: 16) {
            ZStack {
                Circle()
                    .stroke(NymColor.gray1, lineWidth: 2)
                    .frame(width: 64, height: 64)
                GenericImage(systemImageName: "shield.slash")
                    .frame(width: 24, height: 24)
                    .foregroundStyle(NymColor.gray1)
            }
            Text("settings.account.noActivePlan".localizedString)
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Large.regular)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 32)
    }

    func accountStatusHeader() -> some View {
        VStack(spacing: 0) {
            Spacer()
                .frame(height: 16)
            HStack(spacing: 8) {
                GenericImage(systemImageName: "gauge.with.dots.needle.50percent")
                    .frame(width: 20, height: 20)
                    .foregroundStyle(NymColor.gray1)
                Text("settings.account.status".localizedString)
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Body.Large.regular)
                Spacer()
            }
            .padding(.horizontal, 16)
            Spacer()
                .frame(height: 16)
        }
    }

    func accountStatusBandwidth(accountSummary: AccountSummary) -> some View {
        VStack(spacing: 8) {
            HStack {
                Text("settings.account.bandwidthRemaining".localizedString)
                    .foregroundStyle(NymColor.accent)
                    .textStyle(.Body.Small.regular)
                Spacer()
                Text("settings.account.bandwidthLimit".localizedString)
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Small.regular)
            }

            bandwidthProgressBar(
                used: accountSummary.trafficUsedGb,
                limit: accountSummary.trafficLimitGb,
                color: NymColor.accent
            )

            HStack {
                Text(bandwidthRemainingText(used: accountSummary.trafficUsedGb, limit: accountSummary.trafficLimitGb))
                    .foregroundStyle(NymColor.accent)
                    .textStyle(.Body.Small.regular)
                Spacer()
                Text(bandwidthLimitText(limit: accountSummary.trafficLimitGb))
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Small.regular)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    func accountStatusResetDate(accountSummary: AccountSummary) -> some View {
        HStack {
            Text("settings.account.resetsOn".localizedString)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
            Spacer()
            Text(resetDateText(date: accountSummary.trafficResetDate))
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Medium.regular)
        }
        .padding(.horizontal, 16)
        .frame(height: 48)
    }

    @ViewBuilder
    func renewNowRow(accountSummary: AccountSummary) -> some View {
        if !accountSummary.isAutoRenewEnabled {
            let color = accountSummary.statusColor
            Button {
                navigateToPlanPurchase()
            } label: {
                HStack(spacing: 8) {
                    GenericImage(imageName: "bolt")
                        .frame(width: 16, height: 16)
                        .foregroundStyle(color)
                    Text("settings.account.renewNow".localizedString)
                        .foregroundStyle(color)
                        .textStyle(.Body.Medium.regular)
                    Spacer()
                    GenericImage(imageName: "externalLink")
                        .frame(width: 16, height: 16)
                        .foregroundStyle(color)
                }
                .padding(.horizontal, 16)
                .frame(height: 48)
                .background(color.opacity(0.15))
            }
            .buttonStyle(.plain)
        }
    }

    func bandwidthProgressBar(used: Int?, limit: Int?, color: Color) -> some View {
        GeometryReader { geometry in
            let remaining = max(0, (limit ?? 0) - (used ?? 0))
            let fraction = limit.map { $0 > 0 ? CGFloat(remaining) / CGFloat($0) : 0 } ?? 0

            ZStack(alignment: .leading) {
                RoundedRectangle(cornerRadius: 4)
                    .fill(NymColor.gray2)
                    .frame(height: 8)

                RoundedRectangle(cornerRadius: 4)
                    .fill(color)
                    .frame(width: geometry.size.width * fraction, height: 8)
            }
        }
        .frame(height: 8)
    }

    func bandwidthRemainingText(used: Int?, limit: Int?) -> String {
        let remaining = max(0, (limit ?? 0) - (used ?? 0))
        return formatBandwidth(remaining)
    }

    func bandwidthLimitText(limit: Int?) -> String {
        formatBandwidth(limit ?? 0)
    }

    func formatBandwidth(_ gb: Int) -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .decimal
        formatter.maximumFractionDigits = 0
        let formatted = formatter.string(from: NSNumber(value: gb)) ?? "\(gb)"
        return "\(formatted) GB"
    }

    func resetDateText(date: Date?) -> String {
        guard let date else { return "-" }
        let formatter = DateFormatter()
        formatter.locale = .autoupdatingCurrent
        formatter.dateStyle = .long
        formatter.timeStyle = .none
        return formatter.string(from: date)
    }
}
