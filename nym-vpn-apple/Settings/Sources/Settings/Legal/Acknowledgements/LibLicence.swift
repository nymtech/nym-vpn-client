import Foundation

struct LibLicence: Decodable {
    let name: String
    let version: String
    let authors: String?
    let repository: URL?
    let license: String?
    let licenseFile: String?
    let description: String?
}
