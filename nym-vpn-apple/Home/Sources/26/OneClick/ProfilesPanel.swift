import SwiftUI
import ConnectionTypes
import Theme
import UIComponents

/// Popup listing the 4 built-in connection profiles, anchored under the nav bar's leading icon.
struct ProfilesPanel: View {
    let profiles: [ConnectionProfile]
    let selection: ConnectionProfile?
    let onSelect: (ConnectionProfile) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            ForEach(profiles, id: \.self) { profile in
                ProfileRow(
                    imageName: profile.imageName,
                    title: profile.titleKey.localizedString,
                    description: profile.descriptionKey.localizedString,
                    isSelected: profile == selection
                ) {
                    onSelect(profile)
                }
            }
        }
        .padding(NymSpacing.extraExtraSmall)
        .frame(maxWidth: Constants.maxWidth)
        .background(
            RoundedRectangle(cornerRadius: Constants.cornerRadius).fill(Color.Nym.surface)
        )
        .overlay(
            RoundedRectangle(cornerRadius: Constants.cornerRadius)
                .stroke(Color.Nym.textTertiary.opacity(0.2), lineWidth: 1)
        )
        .shadow(color: .black.opacity(0.15), radius: 12, y: 4)
    }

    private enum Constants {
        static let maxWidth: CGFloat = 320
        static let cornerRadius: CGFloat = 16
    }
}
