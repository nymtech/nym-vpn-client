#if os(iOS)
import SwiftUI
import CredentialsManager
import Settings
import Theme

struct IOSPurchaseChromeModifier: ViewModifier {
    let viewModel: AppFeatureViewModel
    let autologinState: AutologinState
    let credentialsManager: CredentialsManager

    func body(content: Content) -> some View {
        @Bindable var viewModel = viewModel
        content
            .confirmationDialog(
                "subscriptionPurchase.choice.title".localizedString,
                isPresented: $viewModel.isSubscriptionPurchaseChoiceDisplayed,
                titleVisibility: .visible
            ) {
                Button("subscriptionPurchase.choice.inApp".localizedString) {
                    viewModel.beginInAppSubscriptionPurchase()
                }
                Button("subscriptionPurchase.choice.web".localizedString) {
                    viewModel.beginWebSubscriptionPurchase()
                }
                Button("cancel".localizedString, role: .cancel) {
                    viewModel.dismissSubscriptionPurchaseChoice()
                }
            } message: {
                Text("subscriptionPurchase.choice.message".localizedString)
            }
            .autologinOverlay(state: autologinState, onRetry: { viewModel.beginWebSubscriptionPurchase() })
            .onChange(of: credentialsManager.didReceiveSubscriptionPayment) { _, received in
                guard received else { return }
                autologinState.dismissAfterWebReturn()
                viewModel.reconcilePurchaseFlowAfterAccountRefresh()
            }
            .onChange(of: viewModel.webSubscriptionPurchaseToken) { _, _ in
                autologinState.start(kind: .autologinRenew, using: credentialsManager)
            }
    }
}
#endif
