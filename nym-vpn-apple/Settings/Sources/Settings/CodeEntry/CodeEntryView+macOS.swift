#if os(macOS)
import SwiftUI

public struct CodeEntryView: View {
    private let onFinish: (CodeEntryOutcome) -> Void

    @State private var redeemViewModel: RedeemCodeViewModel

    public init(onFinish: @escaping (CodeEntryOutcome) -> Void) {
        self.onFinish = onFinish
        _redeemViewModel = State(wrappedValue: RedeemCodeViewModel(validator: ScannedCodeValidator()))
    }

    public var body: some View {
        RedeemCodeView(viewModel: redeemViewModel) {
            onFinish(.cancelled)
        }
        .frame(width: 420)
        .frame(minHeight: 340)
        // A redeem in flight can't be cancelled, so the sheet stays open.
        .interactiveDismissDisabled(redeemViewModel.isLoading)
        .onAppear {
            redeemViewModel.onAccepted = { _ in onFinish(.freepassRedeemed) }
        }
    }
}
#endif
