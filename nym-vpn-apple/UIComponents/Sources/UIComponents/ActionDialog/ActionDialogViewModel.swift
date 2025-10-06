import SwiftUI
import ImpactGenerator

public final class ActionDialogViewModel: ObservableObject {
    let impactGenerator: ImpactGenerator
    let configuration: ActionDialogConfiguration

    @Binding var isDisplayed: Bool

    public init(
        isDisplayed: Binding<Bool>,
        configuration: ActionDialogConfiguration,
        impactGenerator: ImpactGenerator = ImpactGenerator.shared
    ) {
        _isDisplayed = isDisplayed
        self.impactGenerator = impactGenerator
        self.configuration = configuration
    }
}
