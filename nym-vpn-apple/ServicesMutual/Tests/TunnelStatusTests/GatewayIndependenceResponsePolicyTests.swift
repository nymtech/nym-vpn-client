import ErrorReason
import Foundation
import Testing
@testable import TunnelStatus

struct GatewayIndependenceResponsePolicyTests {
    @Test func independenceErrorWithNotificationsShowsModal() {
        let action = GatewayIndependenceResponsePolicy.action(
            status: .error,
            lastError: ErrorReason.needsRelaxedIndependenceCriteria,
            notificationsEnabled: true,
            isHandlingEpisode: false
        )
        #expect(action == .showModal)
    }

    @Test func independenceErrorWithoutNotificationsAutoRelaxes() {
        let action = GatewayIndependenceResponsePolicy.action(
            status: .error,
            lastError: ErrorReason.needsRelaxedIndependenceCriteria,
            notificationsEnabled: false,
            isHandlingEpisode: false
        )
        #expect(action == .autoRelaxAndReconnect)
    }

    @Test func nonIndependenceErrorIsIgnored() {
        let action = GatewayIndependenceResponsePolicy.action(
            status: .error,
            lastError: NSError(domain: "test", code: 1),
            notificationsEnabled: true,
            isHandlingEpisode: false
        )
        #expect(action == .noAction)
    }

    @Test func connectedStatusDoesNotTriggerResponse() {
        let action = GatewayIndependenceResponsePolicy.action(
            status: .connected,
            lastError: ErrorReason.needsRelaxedIndependenceCriteria,
            notificationsEnabled: true,
            isHandlingEpisode: false
        )
        #expect(action == .noAction)
    }

    @Test func duplicateEpisodeIsIgnored() {
        let action = GatewayIndependenceResponsePolicy.action(
            status: .error,
            lastError: ErrorReason.needsRelaxedIndependenceCriteria,
            notificationsEnabled: false,
            isHandlingEpisode: true
        )
        #expect(action == .noAction)
    }

    @Test func clearsHandlingWhenErrorLeavesIndependenceConsent() {
        #expect(
            GatewayIndependenceResponsePolicy.shouldClearHandlingEpisode(
                status: .error,
                lastError: nil
            )
        )
        #expect(
            GatewayIndependenceResponsePolicy.shouldClearHandlingEpisode(
                status: .connected,
                lastError: ErrorReason.needsRelaxedIndependenceCriteria
            )
        )
    }
}
