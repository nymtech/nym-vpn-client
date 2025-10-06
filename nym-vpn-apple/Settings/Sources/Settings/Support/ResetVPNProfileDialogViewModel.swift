import SwiftUI
import ImpactGenerator

final class ResetVPNProfileDialogViewModel: ObservableObject {
    let impactGenerator: ImpactGenerator
    let resetVpnProfileTitle = "settings.resetVpnProfileTitle".localizedString
    let resetVpnProfileSubtitle = "settings.resetVpnProfileSubtitle".localizedString
    let yesLocalizedString = "logs.yes".localizedString
    let noLocalizedString = "logs.no".localizedString

    let action: () -> Void

    @Binding var isDisplayed: Bool

    init(
        isDisplayed: Binding<Bool>,
        impactGenerator: ImpactGenerator = ImpactGenerator.shared,
        action: @escaping () -> Void
    ) {
        _isDisplayed = isDisplayed
        self.impactGenerator = impactGenerator
        self.action = action
    }
}
