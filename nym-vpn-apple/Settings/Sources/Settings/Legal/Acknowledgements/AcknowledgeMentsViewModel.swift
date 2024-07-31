import SwiftUI
import AcknowList

final class AcknowledgeMentsViewModel: ObservableObject {
    let title = "legal.licences".localizedString

    @Published var acknoledgementsList: AcknowList?

    @Binding var path: NavigationPath

    init(path: Binding<NavigationPath>) {
        _path = path
        acknoledgementsList = acknowledgementsList()
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}

extension AcknowledgeMentsViewModel {
    func acknowledgementsList() -> AcknowList? {
        guard let url = Bundle.main.url(forResource: "Package", withExtension: "resolved"),
              let data = try? Data(contentsOf: url),
              let acknowList = try? AcknowPackageDecoder().decode(from: data)
        else {
            return nil
        }
        return acknowList
    }
}
