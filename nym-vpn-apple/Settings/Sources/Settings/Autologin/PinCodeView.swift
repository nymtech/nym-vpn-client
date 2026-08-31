import SwiftUI
import ExternalLinkManager
import ImpactGenerator
import Theme
import UIComponents

struct PinCodeView: View {
    @EnvironmentObject private var externalLinkManager: ExternalLinkManager

    @Binding var isDisplayed: Bool
    @Binding var pinCode: String
    @Binding var url: String
    @State private var didCopy = false

    var body: some View {
        ModalOverlayView(isDisplayed: $isDisplayed) {
            VStack(spacing: 0) {
                closeButton()
                lockIcon()
                title()
                subtitle()
                pinDigits()
                copyCodeButton()
            }
        }
    }
}

// MARK: - Views -
private extension PinCodeView {
    func closeButton() -> some View {
        HStack {
            Spacer()
            Button {
                dismiss()
            } label: {
                Image(systemName: "xmark")
                    .foregroundStyle(Color.Nym.textPrimary)
                    .frame(width: 24, height: 24)
            }
            .buttonStyle(.plain)
            .padding(16)
        }
    }

    func lockIcon() -> some View {
        RoundedRectangle(cornerRadius: 12)
            .fill(Color.Nym.primary.opacity(0.15))
            .frame(width: 56, height: 56)
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(Color.Nym.primary, lineWidth: 1)
            )
            .overlay(
                GenericImage(systemImageName: "lock.fill")
                    .frame(width: 24, height: 24)
                    .foregroundStyle(Color.Nym.primary)
            )
    }

    func title() -> some View {
        VStack(spacing: 0) {
            Spacer()
                .frame(height: 16)

            Text("pinCode.title".localizedString)
                .nymTextStyle(.titleScreen)
                .foregroundStyle(Color.Nym.textPrimary)
        }
    }

    func subtitle() -> some View {
        VStack(spacing: 0) {
            Spacer()
                .frame(height: 8)

            Text("pinCode.subtitle".localizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textSecondary)
        }
    }

    func pinDigits() -> some View {
        VStack(spacing: 0) {
            Spacer()
                .frame(height: 24)

            HStack(spacing: 0) {
                let digits = Array(pinCode)
                ForEach(Array(digits.enumerated()), id: \.offset) { index, digit in
                    if index > 0 {
                        Circle()
                            .fill(Color.Nym.primary)
                            .frame(width: 6, height: 6)
                            .padding(.horizontal, 8)
                    }
                    Text(String(digit))
                        .textStyle(.Headline.ExtraExtraLarge.bold)
                        .foregroundStyle(Color.Nym.textPrimary)
                        .monospacedDigit()
                }
            }
            .minimumScaleFactor(0.5)
            .lineLimit(1)

            Spacer()
                .frame(height: 24)
        }
        .padding(.horizontal, 16)
    }

    func copyCodeButton() -> some View {
        GenericButton(
            title: didCopy ? "pinCode.copied".localizedString : "pinCode.copyCode".localizedString,
            systemImageName: didCopy ? "checkmark" : "doc.on.doc"
        )
        .animation(.easeInOut, value: didCopy)
        .padding(.horizontal, 16)
        .padding(.bottom, 24)
        .onTapGesture {
            copyPinCode()
        }
    }
}

// MARK: - Actions -
private extension PinCodeView {
    func copyPinCode() {
        guard !didCopy else { return }
#if os(iOS)
        UIPasteboard.general.string = pinCode
        ImpactGenerator.shared.impact()
#elseif os(macOS)
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(pinCode, forType: .string)
#endif
        Task { [externalLinkManager, url] in
            do {
                try await externalLinkManager.presentPrivyAuthSession(urlString: url)
                dismiss()
            } catch is CancellationError {
                return
            } catch {
                return
            }
        }
        withAnimation(.easeInOut) {
            didCopy = true
        }
        Task {
            try? await Task.sleep(for: .seconds(3))
            withAnimation(.easeInOut) {
                didCopy = false
            }
        }
    }

    func dismiss() {
        pinCode = ""
        url = ""
        isDisplayed = false
    }
}
