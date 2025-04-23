import SwiftUI
#if os(iOS)
import ImpactGenerator
#endif

final class ResetVPNProfileDialogViewModel: ObservableObject {
#if os(iOS)
    let impactGenerator: ImpactGenerator
#endif
    let resetVpnProfileTitle = "settings.resetVpnProfileTitle"
    let resetVpnProfileSubtitle = "settings.resetVpnProfileSubtitle"
    let yesLocalizedString = "logs.yes"
    let noLocalizedString = "logs.no"

    let action: () -> Void

    @Binding var isDisplayed: Bool

#if os(iOS)
    init(
        isDisplayed: Binding<Bool>,
        impactGenerator: ImpactGenerator = ImpactGenerator.shared,
        action: @escaping () -> Void
    ) {
        _isDisplayed = isDisplayed
        self.impactGenerator = impactGenerator
        self.action = action
    }
#endif
#if os(macOS)
    init(isDisplayed: Binding<Bool>, action: @escaping () -> Void) {
        _isDisplayed = isDisplayed
        self.action = action
    }
#endif
}
