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
            accountCreateSuccessfully
            Spacer()
                .frame(height: 24)
            continueTitle
            Spacer()
                .frame(height: 24)

            successItem(
                title: "createAccount.success.fastAnonymousModeTitle".localizedString,
                subtitle: "createAccount.success.fastAnonymousModeSubtitle".localizedString
            )

            successItem(
                title: "createAccount.success.globalCoverageTitle".localizedString,
                subtitle: "createAccount.success.globalCoverageSubtitle".localizedString
            )
            Spacer()
            selectPlanButton
            skipPlanSelect
        }
    }

    var navbar: some View {
        CustomNavBar(useElevationBackground: true)
    }

    var accountCreateSuccessfully: some View {
        Text("createAccount.success.accountCreateSuccessfully".localizedString)
            .textStyle(.Headline.Medium.regular)
            .foregroundStyle(NymColor.primary)
    }

    var continueTitle: some View {
        Text("createAccount.success.toContinueTitle".localizedString)
            .textStyle(.Headline.Small.regular)
            .foregroundStyle(NymColor.primary)
    }

    var selectPlanButton: some View {
        GenericButton(title: "createAccount.success.selectPlan".localizedString)
            .padding(.bottom, 16)
            .onTapGesture {
                selectPlanAction()
            }
            .accessibilityAction {
                selectPlanAction()
            }
            .confirmationDialog(
                "createAccount.success.choosePlan".localizedString,
                isPresented: $isPlanAlertDisplayed,
                titleVisibility: .visible
            ) {
                ForEach(purchasesManager.products, id: \.id) { plan in
                    Button("\(plan.displayName) (\(plan.displayPrice))") {
                        Task {
                            await purchasePlanAction(with: plan)
                        }
                    }
                }
                Button("cancel".localizedString, role: .cancel) {}
            }
    }

    @ViewBuilder var skipPlanSelect: some View {
        if let skipPlanSelectAttributedString = skipPlanSelectAttributedString() {
            HStack {
                Spacer()
                Text(skipPlanSelectAttributedString)
                    .tint(NymColor.accent)
                    .textStyle(.Headline.Small.regular)
                    .multilineTextAlignment(.center)
                    .foregroundStyle(NymColor.gray1)
                    .padding(.bottom, 24)
                    .environment(\.openURL, OpenURLAction { url in
                        guard url.absoluteString == "skip" else { return .discarded }
                        navigateHome()
                        return .handled
                    })
                Spacer()
            }
        }
    }
}

private extension CreateAccountSuccessView {
    func successItem(title: String, subtitle: String) -> some View {
        HStack(alignment: .top, spacing: 0) {
            GenericImage(systemImageName: "checkmark.circle.fill")
                .frame(width: 24, height: 24)
                .foregroundStyle(NymColor.accent)
                .padding(.trailing, 10)

            VStack(alignment: .leading, spacing: 0) {
                Text(title)
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Body.Medium.regular)
                Text(subtitle)
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Small.regular)
            }
        }
        .padding(.bottom, 16)
    }

    func skipPlanSelectAttributedString() -> AttributedString? {
        let maybeLater = "createAccount.success.maybeLater".localizedString
        let skip = "createAccount.success.skipForNow".localizedString
        let skipLink = "skip"
        return try? AttributedString(markdown: "[\(maybeLater)](\(skipLink)) (\(skip))")
    }
}

// MARK: - Actions -
private extension CreateAccountSuccessView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

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
