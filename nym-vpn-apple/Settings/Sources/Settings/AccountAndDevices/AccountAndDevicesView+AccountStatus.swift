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
                    .overlay(Color.Nym.divider)
                    .padding(.horizontal, 16)
                accountStatusResetDate(accountSummary: accountSummary)
                renewNowRow(accountSummary: accountSummary)
            } else {
                Divider()
                    .frame(height: 1)
                    .overlay(Color.Nym.divider)
                accountStatusInactive()
            }
        }
        .background(Color.Nym.backgroundCard)
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }

    func accountStatusInactive() -> some View {
        VStack(spacing: 16) {
            ZStack {
                Circle()
                    .stroke(Color.Nym.textSecondary, lineWidth: 2)
                    .frame(width: 64, height: 64)
                GenericImage(systemImageName: "shield.slash")
                    .frame(width: 24, height: 24)
                    .foregroundStyle(Color.Nym.textSecondary)
            }
            Text("settings.account.noActivePlan".localizedString)
                .foregroundStyle(Color.Nym.textPrimary)
                .nymTextStyle(.bodyLarge)
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
                    .foregroundStyle(Color.Nym.textSecondary)
                Text("settings.account.status".localizedString)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .nymTextStyle(.bodyLarge)
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
                    .foregroundStyle(Color.Nym.primary)
                    .nymTextStyle(.bodySmall)
                Spacer()
                Text("settings.account.bandwidthLimit".localizedString)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)
            }

            bandwidthProgressBar(
                used: accountSummary.trafficUsedGb,
                limit: accountSummary.trafficLimitGb,
                color: Color.Nym.primary
            )

            HStack {
                Text(bandwidthRemainingText(used: accountSummary.trafficUsedGb, limit: accountSummary.trafficLimitGb))
                    .foregroundStyle(Color.Nym.primary)
                    .nymTextStyle(.bodySmall)
                Spacer()
                Text(bandwidthLimitText(limit: accountSummary.trafficLimitGb))
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    func accountStatusResetDate(accountSummary: AccountSummary) -> some View {
        HStack {
            Text("settings.account.resetsOn".localizedString)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            Spacer()
            Text(resetDateText(date: accountSummary.trafficResetDate))
                .foregroundStyle(Color.Nym.textPrimary)
                .nymTextStyle(.bodyDefault)
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
                        .nymTextStyle(.bodyDefault)
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
                    .fill(Color.Nym.gray2)
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
