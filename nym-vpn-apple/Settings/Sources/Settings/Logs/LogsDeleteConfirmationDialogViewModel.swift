import SwiftUI

final class LogsDeleteConfirmationDialogViewModel: ObservableObject {
    let trashIconImageName = "trash"
    let deleteAllLogsLocalizedString = "logs.deleteAllLogs".localizedString
    let cannotRetrieveLogsLocalizedString = "logs.noRetrieval".localizedString
    let yesLocalizedString = "logs.yes".localizedString
    let noLocalizedString = "logs.no".localizedString

    let action: () -> Void

    @Binding var isDisplayed: Bool

    init(isDisplayed: Binding<Bool>, action: @escaping () -> Void) {
        _isDisplayed = isDisplayed
        self.action = action
    }
}
