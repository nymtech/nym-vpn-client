#if os(iOS)
import SwiftUI
import Theme
import UIComponents

struct PlanOption: Identifiable {
    let id: String
    let title: String
    let subtitle: String?
}

struct SelectPlanSheet: View {
    let title: String
    let options: [PlanOption]
    let cancelTitle: String
    let onSelect: (String) -> Void
    let onCancel: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            Text(title)
                .nymTextStyle(.bodyDefaultBold)
                .foregroundStyle(Color.Nym.textPrimary)
                .multilineTextAlignment(.center)
                .frame(maxWidth: .infinity)

            planList
                .padding(.top, 16)

            NymButton(
                cancelTitle,
                style: .secondary,
                cornerRadius: 28,
                foregroundColor: .Nym.textSecondary,
                borderColor: .Nym.textSecondary,
                action: onCancel
            )
            .padding(.top, 24)
        }
        .padding(20)
    }

    private var planList: some View {
        VStack(spacing: 0) {
            ForEach(Array(options.enumerated()), id: \.element.id) { index, option in
                Button {
                    onSelect(option.id)
                } label: {
                    planRow(option)
                }
                .buttonStyle(.plain)

                if index < options.count - 1 {
                    Rectangle()
                        .fill(Color.Nym.primary.opacity(0.5))
                        .frame(height: 1)
                }
            }
        }
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color.Nym.primary, lineWidth: 1)
        )
    }

    private func planRow(_ option: PlanOption) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(option.title)
                .nymTextStyle(.bodyDefaultBold)
                .foregroundStyle(Color.Nym.textPrimary)

            if let subtitle = option.subtitle {
                Text(subtitle)
                    .nymTextStyle(.bodySmall)
                    .foregroundStyle(Color.Nym.primary)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.vertical, 20)
        .padding(.horizontal, 16)
        .contentShape(Rectangle())
        .accessibilityElement(children: .combine)
    }
}
#endif
