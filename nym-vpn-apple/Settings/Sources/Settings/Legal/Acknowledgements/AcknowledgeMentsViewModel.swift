import SwiftUI
import AcknowList

final class AcknowledgeMentsViewModel: ObservableObject {
    let title = "legal.licences".localizedString

    @Published var acknowledgements = [Acknow]()

    @Binding var navigationPath: NavigationPath

    init(navigationPath: Binding<NavigationPath>) {
        _navigationPath = navigationPath

        setup()
    }

    func navigateBack() {
        if !navigationPath.isEmpty { navigationPath.removeLast() }
    }
}

extension AcknowledgeMentsViewModel {
    func setup() {
        var newAcknowledgements = [Acknow]()
        if let appAcknowledgments = appAcknowledgementsList()?.acknowledgements {
            newAcknowledgements.append(contentsOf: appAcknowledgments)
        }
        if let libLicences = libLicences() {
            newAcknowledgements.append(contentsOf: libLicences)
        }
        newAcknowledgements = newAcknowledgements.sorted(by: { $0.title < $1.title })
        acknowledgements = newAcknowledgements
    }

    func appAcknowledgementsList() -> AcknowList? {
        guard let url = Bundle.main.url(forResource: "Package", withExtension: "resolved"),
              let data = try? Data(contentsOf: url),
              let acknowList = try? AcknowPackageDecoder().decode(from: data)
        else {
            return nil
        }
        return acknowList
    }

    func libLicences() -> [Acknow]? {
        guard let licenceFile = Bundle.main.path(forResource: "LibLicences", ofType: "json"),
              let jsonData = try? String(contentsOfFile: licenceFile).data(using: .utf8),
              let json = try? JSONDecoder().decode([LibLicence].self, from: jsonData)
        else {
            return nil
        }

        return json.compactMap {
            Acknow(title: "\($0.name) (v\($0.version))", license: $0.license, repository: $0.repository)
        }
    }
}
