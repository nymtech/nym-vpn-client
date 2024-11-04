import SwiftUI
#if os(iOS)
import ImpactGenerator
#endif

public final class ActionDialogViewModel: ObservableObject {
#if os(iOS)
    let impactGenerator: ImpactGenerator
#endif
    let configuration: ActionDialogConfiguration

    @Binding var isDisplayed: Bool

#if os(iOS)
    public init(
        isDisplayed: Binding<Bool>,
        configuration: ActionDialogConfiguration,
        impactGenerator: ImpactGenerator = ImpactGenerator.shared
    ) {
        _isDisplayed = isDisplayed
        self.impactGenerator = impactGenerator
        self.configuration = configuration
    }
#endif

#if os(macOS)
    public init(
        isDisplayed: Binding<Bool>,
        configuration: ActionDialogConfiguration
    ) {
        _isDisplayed = isDisplayed
        self.configuration = configuration
    }
#endif
}
