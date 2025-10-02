import SwiftUI
import PurchasesManager
import Theme
import UIComponents

public struct PlanPurchaseSuccessView: View {
    @EnvironmentObject private var purchasesManager: PurchasesManager
    @Binding private var path: NavigationPath
    @State private var isPlanAlertDisplayed = false

    public var body: some View {
        VStack(spacing: 0) {
            VStack(spacing: 0) {
                navbar
                Spacer()
                    .frame(height: 40)
                HStack {
                    VStack(alignment: .center, spacing: 0) {
                        StepView(stepCount: 3, currentStep: 3)
                        Spacer()
                        welcomeToTruePrivacyTitle
                            .padding(.horizontal, 16)
                        Spacer()
                            .frame(height: 24)
                        planActivatedSuccessfullyConnectionPrivate
                        Spacer()
                        startUsingVPNButton
                    }
                    Spacer()
                }
                .padding(.horizontal, 16)

                Spacer()
            }
            Spacer()
                .frame(height: 16)
        }
        .frame(maxWidth: MagicNumbers.moreMaxWidth)

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

private extension PlanPurchaseSuccessView {
    var navbar: some View {
        CustomNavBar(useElevationBackground: true)
    }

    var welcomeToTruePrivacyTitle: some View {
        Text("purchasePlan.welcomeToTruePrivacy".localizedString)
            .foregroundStyle(NymColor.primary)
            .textStyle(.Headline.Large.regular)
            .multilineTextAlignment(.center)
    }

    var planActivatedSuccessfullyConnectionPrivate: some View {
        Text("purchasePlan.fullyProtected".localizedString)
            .foregroundStyle(NymColor.primary)
            .textStyle(.Headline.Small.regular)
    }

    var startUsingVPNButton: some View {
        GenericButton(title: "purchasePlan.startUsing".localizedString)
            .onTapGesture {
                navigateHome()
            }
    }
}

private extension PlanPurchaseSuccessView {
    func navigateHome() {
        path = .init()
    }
}
