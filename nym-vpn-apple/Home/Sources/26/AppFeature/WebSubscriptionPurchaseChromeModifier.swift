#if os(macOS)
import SwiftUI
import CredentialsManager
import Settings
import Theme

struct WebSubscriptionPurchaseChromeModifier: ViewModifier {
    let viewModel: AppFeatureViewModel
    let autologinState: AutologinState
    let credentialsManager: CredentialsManager

    func body(content: Content) -> some View {
        content
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
