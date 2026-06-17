import SwiftUI
import Theme
import UIComponents

public struct NotificationsView: View {
    @StateObject private var viewModel: NotificationsViewModel

    public init(viewModel: NotificationsViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)
            toggleRow()
                .padding(.horizontal, 16)
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
    }
}

private extension NotificationsView {
    func navbar() -> some View {
        CustomNavBar(
            title: "settings.notifications.title".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    func toggleRow() -> some View {
        HStack(alignment: .center, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("settings.notifications.serverFamilyReminders.title".localizedString)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .nymTextStyle(.bodyDefault)
                Text("settings.notifications.serverFamilyReminders.subtitle".localizedString)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)
            }
            Spacer()
            Toggle("", isOn: Binding(
                get: { viewModel.isServerFamilyRemindersEnabled },
                set: { viewModel.setServerFamilyReminders($0) }
            ))
            .toggleStyle(.switch)
            .tint(Color.Nym.primary)
            .labelsHidden()
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.Nym.surface)
        .cornerRadius(12)
    }
}
