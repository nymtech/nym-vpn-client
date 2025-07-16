import SwiftUI
import ConfigurationManager
import CredentialsManager
#if os(iOS)
import ImpactGenerator
import MixnetLibrary
import ErrorHandler
#endif
import PurchasesManager
import StoreKit
import Theme
import UIComponents

public struct CreateAccountSuccessView: View {
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @EnvironmentObject private var purchasesManager: PurchasesManager

    @Binding private var path: NavigationPath

    @State private var isPlanAlertDisplayed = false
    @State private var isDisplayingAlert = false
    @State private var alertTitle = ""

    public var body: some View {
        VStack(spacing: 0) {
            navbar
            Spacer()
                .frame(height: 40)

            VStack(spacing: 0) {
                HStack {
                    content
                    Spacer()
                }
                .padding(.horizontal, 16)
                .alert(alertTitle, isPresented: $isDisplayingAlert) {
                    Button("ok".localizedString, role: .cancel) {}
                }

                Spacer()
                    .frame(height: 16)
            }
            .frame(maxWidth: MagicNumbers.moreMaxWidth)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

private extension CreateAccountSuccessView {
    var content: some View {
        VStack(alignment: .leading, spacing: 0) {
            StepView(stepCount: 3, currentStep: 2)
            Spacer()
                .frame(height: 40)
            checkmarkImage
            Spacer()
                .frame(height: 12)
            titleSubtitleView
            Spacer()
            selectPlanButton
            Spacer()
                .frame(height: 8)
            Spacer()
        }
    }

    var navbar: some View {
        CustomNavBar(useElevationBackground: true)
    }

    var checkmarkImage: some View {
        HStack {
            Spacer()
            ZStack {
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color(red: 0.07, green: 0.77, blue: 0.37).opacity(0.15))
                    .frame(width: 68, height: 68)
                    .overlay(
                        RoundedRectangle(cornerRadius: 8)
                            .stroke(Color(red: 0.08, green: 0.91, blue: 0.44).opacity(0.25), lineWidth: 1)
                    )

                GenericImage(imageName: "checkmarkCircle")
                    .frame(width: 46, height: 46)
            }
            Spacer()
        }
    }
}

private extension CreateAccountSuccessView {
    func skipPlanSelectAttributedString() -> AttributedString? {
        let maybeLater = "createAccount.success.maybeLater".localizedString
        let skip = "createAccount.success.skipForNow".localizedString
        let skipLink = "skip"
        return try? AttributedString(markdown: "[\(maybeLater)](\(skipLink)) (\(skip))")
    }
}

// MARK: - Actions -
private extension CreateAccountSuccessView {
    func navigateHome() {
        path = .init()
    }

    func navigateToDidPurchaseSuccessfully() {
        path.append(SettingLink.planPurchaseSuccess)
    }

    func selectPlanAction() {
        isPlanAlertDisplayed = true
    }

    func purchasePlanAction(with plan: Product) async {
#if os(iOS)
        ImpactGenerator.shared.impact()
#endif
        do {
            guard let token = credentialsManager.accountToken
            else {
                try await credentialsManager.registerAccount()
                return
            }
            let didPurchaseSuccesfully = try await purchasesManager.purchase(
                with: plan,
                token: token
            )
            guard didPurchaseSuccesfully else { return }
            navigateToDidPurchaseSuccessfully()
        } catch {
            Task { @MainActor in
#if os(iOS)
                if let lastVPNError = error as? VpnError {
                    alertTitle = VPNErrorReason(with: lastVPNError).errorDescription ?? ""
                } else {
                    alertTitle = error.localizedDescription
                }
                isDisplayingAlert = true
#endif
            }
        }
    }
}
