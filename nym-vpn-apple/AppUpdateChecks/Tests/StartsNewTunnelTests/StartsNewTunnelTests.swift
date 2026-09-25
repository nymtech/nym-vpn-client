import Testing
import TunnelStatus

struct StartsNewTunnelTests {
    @Test func inProgressOrUpDoesNotStartATunnel() {
        #expect(!TunnelStatus.connected.startsNewTunnel)
        #expect(!TunnelStatus.connecting.startsNewTunnel)
        #expect(!TunnelStatus.error.startsNewTunnel)
        #expect(!TunnelStatus.offlineReconnect.startsNewTunnel)
    }

    @Test func downStatusesStartATunnel() {
        #expect(TunnelStatus.disconnected.startsNewTunnel)
        #expect(TunnelStatus.disconnecting.startsNewTunnel)
        #expect(TunnelStatus.reasserting.startsNewTunnel)
        #expect(TunnelStatus.restarting.startsNewTunnel)
        #expect(TunnelStatus.offline.startsNewTunnel)
        #expect(TunnelStatus.unknown.startsNewTunnel)
    }
}
