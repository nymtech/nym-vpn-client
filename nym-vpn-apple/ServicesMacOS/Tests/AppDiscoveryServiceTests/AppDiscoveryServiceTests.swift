import Foundation
import Testing
@testable import AppDiscoveryService

@Suite
struct AppDiscoveryServiceTests {
    let service = AppDiscoveryService()

    private func makeTempDir() -> URL {
        let url = FileManager.default.temporaryDirectory
            .appendingPathComponent("adstest-\(UUID().uuidString)")
        try? FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }

    @Test
    func bareExecutableFileResolvesToItsOwnPath() throws {
        let bin = makeTempDir().appendingPathComponent("mybin")
        try Data().write(to: bin)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: bin.path)

        let app = service.foundApp(at: bin)

        #expect(app.executablePath == bin.path)
        #expect(app.name == "mybin")
    }

    @Test
    func nonExecutableFileIsRejected() throws {
        let file = makeTempDir().appendingPathComponent("notes.txt")
        try Data().write(to: file)
        try FileManager.default.setAttributes([.posixPermissions: 0o644], ofItemAtPath: file.path)

        let app = service.foundApp(at: file)

        #expect(app.executablePath == nil)
    }

    @Test
    func directoryIsRejected() throws {
        let sub = makeTempDir().appendingPathComponent("somedir")
        try FileManager.default.createDirectory(at: sub, withIntermediateDirectories: true)

        let app = service.foundApp(at: sub)

        #expect(app.executablePath == nil)
    }

    @Test
    func appBundleStillResolvesViaInfoPlist() throws {
        let appURL = makeTempDir().appendingPathComponent("Foo.app")
        let macOS = appURL.appendingPathComponent("Contents/MacOS")
        try FileManager.default.createDirectory(at: macOS, withIntermediateDirectories: true)
        let exec = macOS.appendingPathComponent("Foo")
        try Data().write(to: exec)
        let plist = appURL.appendingPathComponent("Contents/Info.plist")
        let data = try PropertyListSerialization.data(
            fromPropertyList: ["CFBundleExecutable": "Foo"],
            format: .xml,
            options: 0
        )
        try data.write(to: plist)

        let app = service.foundApp(at: appURL)

        #expect(app.executablePath == exec.path)
        #expect(app.name == "Foo")
    }
}
