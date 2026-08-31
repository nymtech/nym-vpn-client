import SwiftUI
import XCTest
@testable import Settings

@MainActor
final class DnsViewModelTests: XCTestCase {
    private func makeViewModel() -> DnsViewModel {
        #if os(macOS)
        DnsViewModel(
            path: .constant(NavigationPath()),
            appSettings: .shared,
            connectionManager: .shared,
            grpcManager: .shared
        )
        #elseif os(iOS)
        DnsViewModel(
            path: .constant(NavigationPath()),
            appSettings: .shared,
            connectionManager: .shared
        )
        #endif
    }

    func testShowsCustomDnsListFalseWhenEmpty() {
        let viewModel = makeViewModel()
        viewModel.customDns = []
        XCTAssertFalse(viewModel.showsCustomDnsList)
    }

    func testShowsCustomDnsListTrueWhenHasEntries() {
        let viewModel = makeViewModel()
        viewModel.customDns = ["1.1.1.1"]
        XCTAssertTrue(viewModel.showsCustomDnsList)
    }

    func testAddValidEntryShowsList() {
        let viewModel = makeViewModel()
        viewModel.customDns = []
        viewModel.customDnsTextField = "1.1.1.1"

        viewModel.add()

        XCTAssertEqual(viewModel.customDns, ["1.1.1.1"])
        XCTAssertTrue(viewModel.showsCustomDnsList)
    }

    func testDeletingLastEntryHidesList() {
        let viewModel = makeViewModel()
        viewModel.customDns = ["1.1.1.1"]

        viewModel.deleteCustom(ipAddr: "1.1.1.1")

        XCTAssertTrue(viewModel.customDns.isEmpty)
        XCTAssertFalse(viewModel.showsCustomDnsList)
    }
}
