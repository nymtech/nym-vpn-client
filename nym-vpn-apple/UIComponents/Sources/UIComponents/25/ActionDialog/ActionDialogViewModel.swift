import SwiftUI
import ImpactGenerator

@MainActor public final class ActionDialogViewModel: ObservableObject {
    let impactGenerator: ImpactGenerator
    let configuration: ActionDialogConfiguration

    @Binding var isDisplayed: Bool
    @Binding var isLoading: Bool

    public init(
        isDisplayed: Binding<Bool>,
        configuration: ActionDialogConfiguration,
        impactGenerator: ImpactGenerator,
        isLoading: Binding<Bool> = .constant(false)
    ) {
        _isDisplayed = isDisplayed
        _isLoading = isLoading
        self.impactGenerator = impactGenerator
        self.configuration = configuration
    }
}
