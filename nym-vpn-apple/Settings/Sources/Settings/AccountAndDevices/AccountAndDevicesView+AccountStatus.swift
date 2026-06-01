import SwiftUI
import ConnectionTypes
import UIComponents
import Theme

// MARK: - Account Status Section -
extension AccountAndDevicesView {
    func accountStatusSection() -> some View {
        VStack(spacing: 0) {
            let isActive = credentialsManager.accountSummary?.isActive == true
            accountStatusHeader(isActive: isActive)
            sectionDivider()
            if let accountSummary = credentialsManager.accountSummary, accountSummary.isActive {
                accountStatusBandwidth(accountSummary: accountSummary)
                sectionDivider()
                accountStatusResetDate(accountSummary: accountSummary)
                if accountSummary.shouldShowRenewRow {
                    sectionDivider()
                    renewNowRow(color: accountSummary.statusColor, isVisible: true)
                }
            } else {
                accountStatusInactive()
                sectionDivider()
                renewNowRow(color: Color.Nym.error, isVisible: true)
            }
        }
        .background(Color.Nym.surface)
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }

    func accountStatusInactive() -> some View {
        VStack(spacing: 16) {
            ZStack {
                Circle()
                    .fill(Color.Nym.error.opacity(0.1))
                    .frame(width: 56, height: 56)
                Circle()
                    .stroke(Color.Nym.error, lineWidth: 1)
                    .frame(width: 56, height: 56)
                GenericImage(systemImageName: "xmark.circle")
                    .frame(width: 24, height: 24)
                    .foregroundStyle(Color.Nym.error)
            }
            Text("settings.account.noActivePlan".localizedString)
                .foregroundStyle(Color.Nym.error)
                .nymTextStyle(.bodyLarge)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 32)
    }

    func accountStatusHeader(isActive: Bool) -> some View {
        VStack(spacing: 0) {
            Spacer()
                .frame(height: 16)
            HStack(spacing: 8) {
                GenericImage(systemImageName: isActive ? "gauge.with.dots.needle.50percent" : "clock")
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
                Text("settings.account.bandwidthUsed".localizedString)
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
                Text(bandwidthUsedText(used: accountSummary.trafficUsedGb))
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
    func renewNowRow(color: Color, isVisible: Bool) -> some View {
        if isVisible {
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
            }
            .buttonStyle(.plain)
        }
    }

    func sectionDivider() -> some View {
        Divider()
            .frame(height: 1)
            .overlay(Color.Nym.divider)
    }

    func bandwidthProgressBar(used: Int?, limit: Int?, color: Color) -> some View {
        GeometryReader { geometry in
            let usedGb = max(0, min(used ?? 0, limit ?? 0))
            let fraction = limit.map { $0 > 0 ? CGFloat(usedGb) / CGFloat($0) : 0 } ?? 0

            ZStack(alignment: .leading) {
                RoundedRectangle(cornerRadius: 4)
                    .fill(Color.Nym.textSecondary)
                    .frame(height: 8)

                RoundedRectangle(cornerRadius: 4)
                    .fill(color)
                    .frame(width: geometry.size.width * fraction, height: 8)
            }
        }
        .frame(height: 8)
    }

    func bandwidthUsedText(used: Int?) -> String {
        formatBandwidth(max(0, used ?? 0))
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
