import SwiftUI
import ImpactGenerator

final class LogsDeleteConfirmationDialogViewModel: ObservableObject {
    let impactGenerator: ImpactGenerator
    let trashIconImageName = "trash"
    let deleteAllLogsLocalizedString = "logs.deleteAllLogs".localizedString
    let cannotRetrieveLogsLocalizedString = "logs.noRetrieval".localizedString
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
