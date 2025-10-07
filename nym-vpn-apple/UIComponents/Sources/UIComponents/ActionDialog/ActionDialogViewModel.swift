import SwiftUI
import ImpactGenerator

@MainActor public final class ActionDialogViewModel: ObservableObject {
    let impactGenerator: ImpactGenerator
    let configuration: ActionDialogConfiguration

    @Binding var isDisplayed: Bool

    public init(
        isDisplayed: Binding<Bool>,
        configuration: ActionDialogConfiguration,
        impactGenerator: ImpactGenerator
    ) {
        _isDisplayed = isDisplayed
        self.impactGenerator = impactGenerator
        self.configuration = configuration
    }
}
