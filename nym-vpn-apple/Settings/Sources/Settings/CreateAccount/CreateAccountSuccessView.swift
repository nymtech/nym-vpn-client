import SwiftUI
import ConfigurationManager
import CredentialsManager
import PurchasesManager
import Theme
import UIComponents

public struct CreateAccountSuccessView: View {
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @EnvironmentObject private var purchasesManager: PurchasesManager

    @Binding private var path: NavigationPath

    @State private var isPlanAlertDisplayed = false

    public var body: some View {
        VStack(spacing: 0) {
            navbar
            Spacer()
                .frame(height: 40)
            HStack {
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
                Spacer()
            }
            .padding(.horizontal, 16)

            Spacer()
                .frame(height: 16)
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
                selectPlan()
            }
            .accessibilityAction {
                selectPlan()
            }
            .confirmationDialog(
                "createAccount.success.choosePlan".localizedString,
                isPresented: $isPlanAlertDisplayed,
                titleVisibility: .visible
            ) {
                ForEach(purchasesManager.products, id: \.id) { plan in
                    Button("\(plan.displayName) (\(plan.displayPrice))") {
                        Task {
                            do {
                                try await purchasesManager.purchase(with: plan, token: credentialsManager.accountToken)
                            }
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

    func selectPlan() {
        isPlanAlertDisplayed = true
    }
}
