import SwiftUI
import ImpactGenerator

@MainActor public final class ActionDialogViewModel: ObservableObject {
    let impactGenerator: ImpactGenerator
    let configuration: ActionDialogConfiguration

    @Binding var isDisplayed: Bool
    @Binding var isLoading: Bool
    @Binding var loadingTextOverride: String?

    public init(
        isDisplayed: Binding<Bool>,
        configuration: ActionDialogConfiguration,
        impactGenerator: ImpactGenerator,
        isLoading: Binding<Bool> = .constant(false),
        loadingTextOverride: Binding<String?> = .constant(nil)
    ) {
        _isDisplayed = isDisplayed
        _isLoading = isLoading
        _loadingTextOverride = loadingTextOverride
        self.impactGenerator = impactGenerator
        self.configuration = configuration
    }

    var displayedLoadingText: String? {
        if let loadingTextOverride, !loadingTextOverride.isEmpty {
            return loadingTextOverride
        }
        return configuration.loadingText
    }
}
