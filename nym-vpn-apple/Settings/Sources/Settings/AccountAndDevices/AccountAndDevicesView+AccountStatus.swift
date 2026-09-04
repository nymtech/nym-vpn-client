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
            } else if credentialsManager.accountSummary != nil {
                accountStatusInactive()
                sectionDivider()
                renewNowRow(color: Color.Nym.error, isVisible: true)
            } else {
                accountStatusPendingSummary()
            }
        }
        .background(Color.Nym.surface)
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }

    func accountStatusInactive() -> some View {
        accountStatusMessage(
            systemImageName: "xmark.circle",
            titleKey: "settings.account.noActivePlan",
            color: Color.Nym.error
        )
    }

    func accountStatusPendingSummary() -> some View {
        let lastFetchFailed = credentialsManager.accountSummaryLastFetchFailed
        let titleKey = lastFetchFailed ? "home.accountUnreachable" : "requestingZkNyms"
        return accountStatusMessage(
            systemImageName: lastFetchFailed ? "exclamationmark.triangle" : "clock",
            titleKey: titleKey,
            color: lastFetchFailed ? Color.Nym.error : Color.Nym.textSecondary
        )
    }

    func accountStatusMessage(systemImageName: String, titleKey: String, color: Color) -> some View {
        VStack(spacing: 16) {
            ZStack {
                Circle()
                    .fill(color.opacity(0.1))
                    .frame(width: 56, height: 56)
                Circle()
                    .stroke(color, lineWidth: 1)
                    .frame(width: 56, height: 56)
                GenericImage(systemImageName: systemImageName)
                    .frame(width: 24, height: 24)
                    .foregroundStyle(color)
            }
            Text(titleKey.localizedString)
                .foregroundStyle(color)
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
                refreshAccountButton()
            }
            .padding(.horizontal, 16)
            Spacer()
                .frame(height: 16)
        }
    }

    @ViewBuilder
    func refreshAccountButton() -> some View {
        Button {
            refreshAccount()
        } label: {
            if isRefreshingAccount {
                ProgressView()
                    .controlSize(.small)
                    .tint(Color.Nym.textSecondary)
                    .frame(width: 20, height: 20)
            } else {
                GenericImage(systemImageName: "arrow.clockwise")
                    .frame(width: 20, height: 20)
                    .foregroundStyle(Color.Nym.textSecondary)
            }
        }
        .buttonStyle(.plain)
        .disabled(isRefreshingAccount)
        .accessibilityLabel("settings.account.refresh".localizedString)
    }

    @ViewBuilder
    func accountStatusBandwidth(accountSummary: AccountSummary) -> some View {
        VStack(spacing: 8) {
            HStack {
                Text("settings.account.dailyAllowanceUsed".localizedString)
                    .foregroundStyle(Color.Nym.primary)
                    .nymTextStyle(.bodySmall)
                Spacer()
                Text("settings.account.dailyLimit".localizedString)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)
            }

            if accountSummary.dataUnavailable {
                accountStatusBandwidthUnavailable()
            } else {
                accountStatusBandwidthDetail(accountSummary: accountSummary)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    func accountStatusBandwidthDetail(accountSummary: AccountSummary) -> some View {
        VStack(spacing: 8) {
            bandwidthProgressBar(
                used: accountSummary.trafficUsedGb,
                limit: accountSummary.trafficLimitGb,
                color: Color.Nym.primary
            )

            HStack {
                Text(bandwidthUsedText(used: accountSummary.trafficUsedGb))
                    .foregroundStyle(Color.Nym.textPrimary)
                    .nymTextStyle(.bodyDefault)
                Spacer()
            }

            HStack {
                Text("settings.account.dailyAllowanceHelper".localizedString)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)
                Spacer()
            }
        }
    }

    func accountStatusBandwidthUnavailable() -> some View {
        HStack(spacing: 8) {
            GenericImage(systemImageName: "exclamationmark.triangle")
                .frame(width: 16, height: 16)
                .foregroundStyle(Color.Nym.textSecondary)
            Text("settings.account.usageDataUnavailable".localizedString)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodySmall)
            Spacer()
        }
        .padding(.vertical, 4)
    }

    func accountStatusResetDate(accountSummary: AccountSummary) -> some View {
        HStack {
            Text("settings.account.resetsDailyUtc".localizedString)
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
                    Text(
                        credentialsManager.accountSummary?.renewButtonTitle
                            ?? "purchasePlan.chooseMyPlan".localizedString
                    )
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

    func formatBandwidth(_ gb: Int) -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .decimal
        formatter.maximumFractionDigits = 0
        let formatted = formatter.string(from: NSNumber(value: gb)) ?? "\(gb)"
        return "\(formatted) GB"
    }

    /// Daily reset is a midnight-UTC boundary, so render it in UTC rather than the
    /// device locale. When the core could not parse `resetsOnUtc` the date is nil and
    /// we show a neutral placeholder that implies no billing-period reset.
    func resetDateText(date: Date?) -> String {
        guard let date else { return "~~" }
        let formatter = DateFormatter()
        formatter.locale = .autoupdatingCurrent
        formatter.timeZone = TimeZone(identifier: "UTC")
        formatter.dateStyle = .long
        formatter.timeStyle = .none
        return formatter.string(from: date)
    }
}
