import SwiftUI
import ImpactGenerator
import Theme
import UIComponents

struct RedeemCodeView: View {
    @Bindable var viewModel: RedeemCodeViewModel
    let onClose: () -> Void

    @FocusState private var isFocused: Bool

    var body: some View {
        VStack(spacing: 0) {
            navbar
            ScrollView {
                form
                    .padding(.horizontal, NymSpacing.section)
                    .padding(.vertical, NymSpacing.section)
            }
            .scrollIndicators(.never)
#if os(iOS)
            .scrollDismissesKeyboard(.interactively)
#endif
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
        .onAppear { isFocused = true }
        .onChange(of: viewModel.errorMessage) { _, message in
            guard let message else { return }
            AccessibilityNotification.Announcement(message).post()
        }
    }
}

private extension RedeemCodeView {
    var navbar: some View {
        HStack {
            Text("redeemCode.title".localizedString)
                .nymTextStyle(.titleScreen)
                .foregroundStyle(Color.Nym.textPrimary)
                .accessibilityAddTraits(.isHeader)
            Spacer()
            Button {
                ImpactGenerator.shared.softImpact()
                onClose()
            } label: {
                Image(systemName: "xmark")
                    .font(.system(size: 18, weight: .medium))
                    .foregroundStyle(Color.Nym.textPrimary)
                    .frame(width: 44, height: 44)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityLabel("close".localizedString)
            // A redeem in flight can't be cancelled, so the screen stays open.
            .disabled(viewModel.isLoading)
        }
        .padding(.leading, NymSpacing.section)
        .padding(.trailing, NymSpacing.medium)
        .padding(.top, NymSpacing.medium)
    }

    var form: some View {
        VStack(alignment: .leading, spacing: NymSpacing.large) {
            Text("redeemCode.description".localizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textSecondary)
            codeField
            if let errorMessage = viewModel.errorMessage {
                Text(errorMessage)
                    .nymTextStyle(.bodySmall)
                    .foregroundStyle(Color.Nym.error)
                    .fixedSize(horizontal: false, vertical: true)
            }
            submitButton
        }
    }

    var codeField: some View {
        VStack(alignment: .leading, spacing: NymSpacing.small) {
            Text("redeemCode.label".localizedString)
                .nymTextStyle(.bodySmall)
                .foregroundStyle(viewModel.errorMessage == nil ? Color.Nym.textPrimary : Color.Nym.error)
                .accessibilityHidden(true)
            TextField("redeemCode.placeholder".localizedString, text: $viewModel.code)
                .nymTextStyle(.bodyLarge)
                .foregroundStyle(Color.Nym.textPrimary)
#if os(iOS)
                .keyboardType(.asciiCapable)
                .textInputAutocapitalization(.never)
#elseif os(macOS)
                .textFieldStyle(.plain)
#endif
                .autocorrectionDisabled()
                .submitLabel(.done)
                .focused($isFocused)
                .disabled(viewModel.isLoading)
                .onSubmit { submit() }
                .padding(NymSpacing.large)
                .overlay {
                    RoundedRectangle(cornerRadius: 12)
                        .stroke(viewModel.errorMessage == nil ? Color.Nym.border : Color.Nym.error, lineWidth: 1)
                }
                .accessibilityLabel("redeemCode.label".localizedString)
        }
    }

    @ViewBuilder
    var submitButton: some View {
        if viewModel.isLoading {
            ZStack {
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color.Nym.textTertiary)
                    .frame(height: 45)
                ProgressView()
                    .tint(Color.Nym.surface)
            }
            .accessibilityElement()
            .accessibilityLabel("redeemCode.checking".localizedString)
        } else {
            NymButton(
                "redeemCode.submit".localizedString,
                style: .primary,
                isDisabled: !viewModel.canSubmit
            ) {
                submit()
            }
        }
    }

    func submit() {
        guard viewModel.canSubmit else { return }
        ImpactGenerator.shared.softImpact()
        isFocused = false
        viewModel.submit()
    }
}
