import XCTest
import SwiftyPing

final class NymVPNUITests: XCTestCase {
    func testPing() throws {
        let app = XCUIApplication()
        app.launch()

        // Connect
        let connectButton = app.staticTexts["Connect"]
        XCTAssertTrue(connectButton.waitForExistence(timeout: 15))
        connectButton.tap()

        // Verify connection
        let connectedText = app.staticTexts["Connected"]
        XCTAssertTrue(connectedText.waitForExistence(timeout: 20))

        // Ping
        let expectedPing = expectation(description: "ping happened")
        var fulfillmentCount = 0

        let pinger = try? SwiftyPing(
            host: "8.8.8.8",
            configuration: PingConfiguration(interval: 1.0),
            queue: DispatchQueue.global(qos: .background)
        )
        pinger?.observer = { response in
            let duration = response.duration
            print("🥳🥳🥳 Ping duration: \(duration)")
            print("response: \(response)")
            fulfillmentCount += 1
            expectedPing.fulfill()
            pinger?.stopPinging()
        }
        try? pinger?.startPinging()

        wait(for: [expectedPing], timeout: 15)
        XCTAssertTrue(fulfillmentCount > 0)

        // Disconnect
        let disconnectButton = app.staticTexts["Disconnect"]
        XCTAssertTrue(disconnectButton.exists)
        disconnectButton.tap()
    }
}
